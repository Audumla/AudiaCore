use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

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

fn create_temporary_file(path: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..TEMP_CREATE_ATTEMPTS {
        let temp = next_temporary_path(path)?;
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
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

struct TempGuard(Option<PathBuf>);

impl TempGuard {
    fn new(path: PathBuf) -> Self {
        Self(Some(path))
    }

    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for TempGuard {
    fn drop(&mut self) {
        if let Some(path) = self.0.as_ref() {
            let _ = fs::remove_file(path);
        }
    }
}

pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let (temp, mut file) = create_temporary_file(path)?;
    let mut guard = TempGuard::new(temp.clone());
    file.write_all(bytes)
        .map_err(|source| io_error("write temporary file", &temp, source))?;
    file.sync_all()
        .map_err(|source| io_error("sync temporary file", &temp, source))?;
    drop(file);

    fs::rename(&temp, path).map_err(|source| io_error("replace destination", path, source))?;
    guard.disarm();

    #[cfg(unix)]
    {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path has no parent: {path:?}"),
            )
        })?;
        let directory =
            File::open(parent).map_err(|source| io_error("open parent directory", parent, source))?;
        directory
            .sync_all()
            .map_err(|source| io_error("sync parent directory", parent, source))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "audiacore-native-file-store-{}-{name}",
            std::process::id()
        ))
    }

    #[test]
    fn atomic_write_creates_and_replaces_file() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("replace");
        let path = root.join("state.bin");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        write_atomic(&path, b"one").unwrap();
        write_atomic(&path, b"two").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"two");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn temporary_name_collision_is_preserved_and_retried() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("collision");
        let path = root.join("state.bin");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let next_id = TEMP_COUNTER.load(Ordering::Relaxed);
        let collision = temporary_path(&path, next_id).unwrap();
        fs::write(&collision, b"owned elsewhere").unwrap();

        write_atomic(&path, b"state").unwrap();

        assert_eq!(fs::read(&collision).unwrap(), b"owned elsewhere");
        assert_eq!(fs::read(&path).unwrap(), b"state");
        fs::remove_dir_all(root).unwrap();
    }
}
