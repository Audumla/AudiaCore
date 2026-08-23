use std::{
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use cap_std::fs::{Dir, File, OpenOptions};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);
const TEMP_CREATE_ATTEMPTS: usize = 32;

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> io::Error {
    io::Error::new(source.kind(), format!("{operation} {path:?}: {source}"))
}

fn temporary_path(path: &Path, id: u64) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path has no file name: {path:?}"),
        )
    })?;
    let mut temp_name = OsString::from(".");
    temp_name.push(name);
    temp_name.push(format!(".tmp-{}-{id}", std::process::id()));
    Ok(path.with_file_name(temp_name))
}

fn next_temporary_path(path: &Path) -> io::Result<PathBuf> {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    temporary_path(path, id)
}

fn create_temporary_file(dir: &Dir, path: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..TEMP_CREATE_ATTEMPTS {
        let temp = next_temporary_path(path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        match dir.open_with(&temp, &options) {
            Ok(file) => return Ok((temp, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(io_error("create temporary file", &temp, source)),
        }
    }

    Err(io_error(
        "create temporary file",
        path,
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "exhausted temporary file name attempts",
        ),
    ))
}

struct TempGuard<'a> {
    dir: &'a Dir,
    path: Option<PathBuf>,
}

impl<'a> TempGuard<'a> {
    fn new(dir: &'a Dir, path: PathBuf) -> Self {
        Self {
            dir,
            path: Some(path),
        }
    }

    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for TempGuard<'_> {
    fn drop(&mut self) {
        if let Some(path) = self.path.as_ref() {
            let _ = self.dir.remove_file(path);
        }
    }
}

#[cfg(unix)]
fn sync_parent_directory(dir: &Dir, path: &Path) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path has no parent: {path:?}"),
        )
    })?;
    let parent_dir = if parent.as_os_str().is_empty() {
        dir.try_clone()
            .map_err(|source| io_error("clone authority directory", parent, source))?
    } else {
        dir.open_dir(parent)
            .map_err(|source| io_error("open parent directory", parent, source))?
    };
    parent_dir
        .into_std_file()
        .sync_all()
        .map_err(|source| io_error("sync parent directory", parent, source))
}

pub(super) fn write_atomic(dir: &Dir, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let (temp, mut file) = create_temporary_file(dir, path)?;
    let mut guard = TempGuard::new(dir, temp.clone());
    file.write_all(bytes)
        .map_err(|source| io_error("write temporary file", &temp, source))?;
    file.sync_all()
        .map_err(|source| io_error("sync temporary file", &temp, source))?;
    drop(file);

    dir.rename(&temp, dir, path)
        .map_err(|source| io_error("replace destination", path, source))?;
    guard.disarm();

    #[cfg(unix)]
    sync_parent_directory(dir, path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_std::{ambient_authority, fs::Dir};
    use std::{fs, sync::Mutex};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "audiacore-native-file-store-{}-{name}",
            std::process::id()
        ))
    }

    fn open_root(path: &Path) -> Dir {
        Dir::open_ambient_dir(path, ambient_authority()).unwrap()
    }

    #[test]
    fn atomic_write_creates_and_replaces_file() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("replace");
        let path = Path::new("state.bin");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let dir = open_root(&root);

        write_atomic(&dir, path, b"one").unwrap();
        write_atomic(&dir, path, b"two").unwrap();
        assert_eq!(fs::read(root.join(path)).unwrap(), b"two");

        drop(dir);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn temporary_name_collision_is_preserved_and_retried() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("collision");
        let path = Path::new("state.bin");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let dir = open_root(&root);

        let next_id = TEMP_COUNTER.load(Ordering::Relaxed);
        let collision = temporary_path(path, next_id).unwrap();
        fs::write(root.join(&collision), b"owned elsewhere").unwrap();

        write_atomic(&dir, path, b"state").unwrap();

        assert_eq!(fs::read(root.join(&collision)).unwrap(), b"owned elsewhere");
        assert_eq!(fs::read(root.join(path)).unwrap(), b"state");
        drop(dir);
        fs::remove_dir_all(root).unwrap();
    }
}
