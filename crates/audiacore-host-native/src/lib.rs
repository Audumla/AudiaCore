//! Native implementations of narrow AudiaCore host contracts.
//!
//! Native effects live here, below reusable application capabilities. This
//! crate does not own policy, configuration, provider, session, or runtime
//! semantics.

mod file_store;
mod process;

pub use process::{NativeProcess, NativeProcessError, NativeProcessHost};

use std::{
    error::Error,
    fmt, io,
    path::{Component, Path, PathBuf},
};

use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};
use cap_std::{ambient_authority, fs::Dir};

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFileHost;

#[derive(Debug)]
pub enum NativeHostError {
    OpenAuthorityRoot { path: PathBuf, source: io::Error },
    AuthorityRootNotDirectory(PathBuf),
    InspectTarget { path: PathBuf, source: io::Error },
    OutsideAuthority { root: PathBuf, path: PathBuf },
    MissingFileName(PathBuf),
    SymbolicLinkWriteTarget(PathBuf),
    DirectoryWriteTarget(PathBuf),
    ReadFile { path: PathBuf, source: io::Error },
    WriteFile { path: PathBuf, source: io::Error },
    RemoveFile { path: PathBuf, source: io::Error },
}

impl fmt::Display for NativeHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAuthorityRoot { path, .. } => {
                write!(f, "cannot open file authority root {path:?}")
            }
            Self::AuthorityRootNotDirectory(path) => {
                write!(f, "file authority root is not a directory: {path:?}")
            }
            Self::InspectTarget { path, .. } => write!(f, "cannot inspect file target {path:?}"),
            Self::OutsideAuthority { root, path } => {
                write!(f, "file target {path:?} is outside authority root {root:?}")
            }
            Self::MissingFileName(path) => write!(f, "file target has no file name: {path:?}"),
            Self::SymbolicLinkWriteTarget(path) => {
                write!(f, "managed write target is a symbolic link: {path:?}")
            }
            Self::DirectoryWriteTarget(path) => {
                write!(f, "managed write target is a directory: {path:?}")
            }
            Self::ReadFile { path, .. } => write!(f, "cannot read file {path:?}"),
            Self::WriteFile { path, .. } => write!(f, "cannot atomically write file {path:?}"),
            Self::RemoveFile { path, .. } => write!(f, "cannot remove file {path:?}"),
        }
    }
}

impl Error for NativeHostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenAuthorityRoot { source, .. }
            | Self::InspectTarget { source, .. }
            | Self::ReadFile { source, .. }
            | Self::WriteFile { source, .. }
            | Self::RemoveFile { source, .. } => Some(source),
            Self::AuthorityRootNotDirectory(_)
            | Self::OutsideAuthority { .. }
            | Self::MissingFileName(_)
            | Self::SymbolicLinkWriteTarget(_)
            | Self::DirectoryWriteTarget(_) => None,
        }
    }
}

fn open_authority_root(root: &Path) -> Result<Dir, NativeHostError> {
    match Dir::open_ambient_dir(root, ambient_authority()) {
        Ok(dir) => Ok(dir),
        Err(source) if source.kind() == io::ErrorKind::NotADirectory => Err(
            NativeHostError::AuthorityRootNotDirectory(root.to_path_buf()),
        ),
        Err(source) => Err(NativeHostError::OpenAuthorityRoot {
            path: root.to_path_buf(),
            source,
        }),
    }
}

fn outside_authority(root: &Path, path: &Path) -> NativeHostError {
    NativeHostError::OutsideAuthority {
        root: root.to_path_buf(),
        path: path.to_path_buf(),
    }
}

/// Convert caller path syntax into a normalized path relative to the granted
/// root. This is a semantic pre-check only; `cap_std::fs::Dir` remains the
/// effect-time containment boundary for every filesystem operation.
fn relative_target(root: &Path, path: &Path) -> Result<PathBuf, NativeHostError> {
    let candidate = if path.is_absolute() {
        path.strip_prefix(root)
            .map_err(|_| outside_authority(root, path))?
    } else {
        path
    };

    let mut relative = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => relative.push(part),
            Component::ParentDir => {
                if !relative.pop() {
                    return Err(outside_authority(root, path));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(outside_authority(root, path));
            }
        }
    }
    Ok(relative)
}

fn display_target(root: &Path, relative: &Path) -> PathBuf {
    root.join(relative)
}

fn ensure_parent_directory(
    dir: &Dir,
    relative: &Path,
    display: &Path,
) -> Result<(), NativeHostError> {
    let parent = relative
        .parent()
        .ok_or_else(|| NativeHostError::MissingFileName(display.to_path_buf()))?;
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    dir.open_dir(parent)
        .map(|_| ())
        .map_err(|source| NativeHostError::InspectTarget {
            path: display.to_path_buf(),
            source,
        })
}

fn inspect_write_target(dir: &Dir, relative: &Path, display: &Path) -> Result<(), NativeHostError> {
    if relative.file_name().is_none() {
        return Err(NativeHostError::MissingFileName(display.to_path_buf()));
    }

    match dir.symlink_metadata(relative) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(
            NativeHostError::SymbolicLinkWriteTarget(display.to_path_buf()),
        ),
        Ok(metadata) if metadata.is_dir() => {
            Err(NativeHostError::DirectoryWriteTarget(display.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            ensure_parent_directory(dir, relative, display)
        }
        Err(source) => Err(NativeHostError::InspectTarget {
            path: display.to_path_buf(),
            source,
        }),
    }
}

impl FileHost for NativeFileHost {
    type Error = NativeHostError;

    fn read_optional(
        &self,
        authority: &FileReadAuthority,
        path: &Path,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let dir = open_authority_root(authority.root())?;
        let relative = relative_target(authority.root(), path)?;
        let display = display_target(authority.root(), &relative);

        match dir.symlink_metadata(&relative) {
            Ok(_) => {
                let read = dir.read(&relative);
                let bytes = read.map_err(|source| NativeHostError::ReadFile {
                    path: display,
                    source,
                })?;
                Ok(Some(bytes))
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                ensure_parent_directory(&dir, &relative, &display)?;
                Ok(None)
            }
            Err(source) => Err(NativeHostError::InspectTarget {
                path: display,
                source,
            }),
        }
    }

    fn write(
        &self,
        authority: &FileWriteAuthority,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        let dir = open_authority_root(authority.root())?;
        let relative = relative_target(authority.root(), path)?;
        let display = display_target(authority.root(), &relative);
        inspect_write_target(&dir, &relative, &display)?;
        file_store::write_atomic(&dir, &relative, bytes).map_err(|source| {
            NativeHostError::WriteFile {
                path: display,
                source,
            }
        })
    }

    fn remove(&self, authority: &FileWriteAuthority, path: &Path) -> Result<(), Self::Error> {
        let dir = open_authority_root(authority.root())?;
        let relative = relative_target(authority.root(), path)?;
        let display = display_target(authority.root(), &relative);
        inspect_write_target(&dir, &relative, &display)?;
        dir.remove_file(&relative)
            .map_err(|source| NativeHostError::RemoveFile {
                path: display,
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::Mutex};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "audiacore-host-native-{}-{name}",
            std::process::id()
        ))
    }

    #[test]
    fn optional_read_write_overwrite_and_remove_are_authority_mediated() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("round-trip");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        let write = FileWriteAuthority::new(&root).unwrap();
        let relative = Path::new("state.bin");

        assert_eq!(host.read_optional(&read, relative).unwrap(), None);
        host.write(&write, relative, b"one").unwrap();
        host.write(&write, relative, b"two").unwrap();
        assert_eq!(
            host.read_optional(&read, relative).unwrap(),
            Some(b"two".to_vec())
        );
        host.remove(&write, relative).unwrap();
        assert_eq!(host.read_optional(&read, relative).unwrap(), None);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn absolute_target_beneath_root_uses_the_same_authority() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("absolute");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let target = root.join("state.bin");

        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        let write = FileWriteAuthority::new(&root).unwrap();
        host.write(&write, &target, b"absolute").unwrap();
        assert_eq!(
            host.read_optional(&read, &target).unwrap(),
            Some(b"absolute".to_vec())
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parent_escape_is_rejected_for_read_write_and_remove() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("escape-root");
        let outside = test_root("escape-outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("state.bin"), b"outside").unwrap();

        let escaped = PathBuf::from("..")
            .join(outside.file_name().unwrap())
            .join("state.bin");
        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        let write = FileWriteAuthority::new(&root).unwrap();

        assert!(matches!(
            host.read_optional(&read, &escaped).unwrap_err(),
            NativeHostError::OutsideAuthority { .. }
        ));
        assert!(matches!(
            host.write(&write, &escaped, b"blocked").unwrap_err(),
            NativeHostError::OutsideAuthority { .. }
        ));
        assert!(matches!(
            host.remove(&write, &escaped).unwrap_err(),
            NativeHostError::OutsideAuthority { .. }
        ));
        assert_eq!(fs::read(outside.join("state.bin")).unwrap(), b"outside");

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn missing_leaf_requires_a_valid_existing_parent() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("missing-parent");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        assert!(matches!(
            host.read_optional(&read, Path::new("missing/state.bin"))
                .unwrap_err(),
            NativeHostError::InspectTarget { .. }
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authority_root_must_exist_as_a_directory_at_effect_time() {
        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("root-file");
        let _ = fs::remove_file(&root);
        let _ = fs::remove_dir_all(&root);
        fs::write(&root, b"not a directory").unwrap();

        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        assert!(matches!(
            host.read_optional(&read, Path::new("state.bin"))
                .unwrap_err(),
            NativeHostError::AuthorityRootNotDirectory(_)
        ));

        fs::remove_file(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn directory_symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("symlink-root");
        let outside = test_root("symlink-outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("state.bin"), b"outside").unwrap();
        symlink(&outside, root.join("escape")).unwrap();

        let host = NativeFileHost;
        let read = FileReadAuthority::new(&root).unwrap();
        let write = FileWriteAuthority::new(&root).unwrap();
        let escaped = root.join("escape").join("state.bin");

        assert!(host.read_optional(&read, &escaped).is_err());
        assert!(host.write(&write, &escaped, b"blocked").is_err());
        assert_eq!(fs::read(outside.join("state.bin")).unwrap(), b"outside");

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_leaf_is_never_replaced_or_removed() {
        use std::os::unix::fs::symlink;

        let _guard = TEST_LOCK.lock().unwrap();
        let root = test_root("leaf-root");
        let outside = test_root("leaf-outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let outside_file = outside.join("state.bin");
        fs::write(&outside_file, b"outside").unwrap();
        let link = root.join("state.bin");
        symlink(&outside_file, &link).unwrap();

        let host = NativeFileHost;
        let write = FileWriteAuthority::new(&root).unwrap();
        assert!(matches!(
            host.write(&write, &link, b"blocked").unwrap_err(),
            NativeHostError::SymbolicLinkWriteTarget(_)
        ));
        assert!(matches!(
            host.remove(&write, &link).unwrap_err(),
            NativeHostError::SymbolicLinkWriteTarget(_)
        ));
        assert_eq!(fs::read(outside_file).unwrap(), b"outside");

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }
}
