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
    fmt, fs, io,
    path::{Path, PathBuf},
};

use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFileHost;

#[derive(Debug)]
pub enum NativeHostError {
    CanonicalizeAuthorityRoot { path: PathBuf, source: io::Error },
    AuthorityRootNotDirectory(PathBuf),
    InspectTarget { path: PathBuf, source: io::Error },
    CanonicalizeTarget { path: PathBuf, source: io::Error },
    CanonicalizeParent { path: PathBuf, source: io::Error },
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
            Self::CanonicalizeAuthorityRoot { path, .. } => {
                write!(f, "cannot canonicalize file authority root {path:?}")
            }
            Self::AuthorityRootNotDirectory(path) => {
                write!(f, "file authority root is not a directory: {path:?}")
            }
            Self::InspectTarget { path, .. } => write!(f, "cannot inspect file target {path:?}"),
            Self::CanonicalizeTarget { path, .. } => {
                write!(f, "cannot canonicalize file target {path:?}")
            }
            Self::CanonicalizeParent { path, .. } => {
                write!(f, "cannot canonicalize file target parent {path:?}")
            }
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
            Self::CanonicalizeAuthorityRoot { source, .. }
            | Self::InspectTarget { source, .. }
            | Self::CanonicalizeTarget { source, .. }
            | Self::CanonicalizeParent { source, .. }
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

fn canonical_authority_root(root: &Path) -> Result<PathBuf, NativeHostError> {
    let canonical =
        fs::canonicalize(root).map_err(|source| NativeHostError::CanonicalizeAuthorityRoot {
            path: root.to_path_buf(),
            source,
        })?;
    if !canonical.is_dir() {
        return Err(NativeHostError::AuthorityRootNotDirectory(canonical));
    }
    Ok(canonical)
}

fn requested_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn ensure_inside(root: &Path, path: &Path) -> Result<(), NativeHostError> {
    if path == root || path.starts_with(root) {
        Ok(())
    } else {
        Err(NativeHostError::OutsideAuthority {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    }
}

fn authorize_optional_read(
    authority: &FileReadAuthority,
    path: &Path,
) -> Result<Option<PathBuf>, NativeHostError> {
    let root = canonical_authority_root(authority.root())?;
    let requested = requested_path(authority.root(), path);

    match fs::symlink_metadata(&requested) {
        Ok(_) => {
            let canonical = fs::canonicalize(&requested).map_err(|source| {
                NativeHostError::CanonicalizeTarget {
                    path: requested.clone(),
                    source,
                }
            })?;
            ensure_inside(&root, &canonical)?;
            Ok(Some(canonical))
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            let parent = requested
                .parent()
                .ok_or_else(|| NativeHostError::MissingFileName(requested.clone()))?;
            let canonical_parent =
                fs::canonicalize(parent).map_err(|source| NativeHostError::CanonicalizeParent {
                    path: parent.to_path_buf(),
                    source,
                })?;
            ensure_inside(&root, &canonical_parent)?;
            Ok(None)
        }
        Err(source) => Err(NativeHostError::InspectTarget {
            path: requested,
            source,
        }),
    }
}

fn authorize_write(
    authority: &FileWriteAuthority,
    path: &Path,
) -> Result<PathBuf, NativeHostError> {
    let root = canonical_authority_root(authority.root())?;
    let requested = requested_path(authority.root(), path);
    let file_name = requested
        .file_name()
        .ok_or_else(|| NativeHostError::MissingFileName(requested.clone()))?;
    let parent = requested
        .parent()
        .ok_or_else(|| NativeHostError::MissingFileName(requested.clone()))?;
    let canonical_parent =
        fs::canonicalize(parent).map_err(|source| NativeHostError::CanonicalizeParent {
            path: parent.to_path_buf(),
            source,
        })?;
    ensure_inside(&root, &canonical_parent)?;

    let target = canonical_parent.join(file_name);
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(NativeHostError::SymbolicLinkWriteTarget(target))
        }
        Ok(metadata) if metadata.is_dir() => Err(NativeHostError::DirectoryWriteTarget(target)),
        Ok(_) => Ok(target),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(target),
        Err(source) => Err(NativeHostError::InspectTarget {
            path: target,
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
        let Some(path) = authorize_optional_read(authority, path)? else {
            return Ok(None);
        };
        fs::read(&path)
            .map(Some)
            .map_err(|source| NativeHostError::ReadFile { path, source })
    }

    fn write(
        &self,
        authority: &FileWriteAuthority,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        let path = authorize_write(authority, path)?;
        file_store::write_atomic(&path, bytes)
            .map_err(|source| NativeHostError::WriteFile { path, source })
    }

    fn remove(&self, authority: &FileWriteAuthority, path: &Path) -> Result<(), Self::Error> {
        let path = authorize_write(authority, path)?;
        fs::remove_file(&path).map_err(|source| NativeHostError::RemoveFile { path, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

        assert!(matches!(
            host.read_optional(&read, &escaped).unwrap_err(),
            NativeHostError::OutsideAuthority { .. }
        ));
        assert!(matches!(
            host.write(&write, &escaped, b"blocked").unwrap_err(),
            NativeHostError::OutsideAuthority { .. }
        ));
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
