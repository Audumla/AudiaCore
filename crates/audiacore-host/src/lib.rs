//! Narrow host-facility contracts with explicit authority scopes.
//!
//! This crate defines permission-bearing boundaries only. It never performs
//! native I/O and does not aggregate facilities into a global host object.

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};

const FILE_ROOT_NOT_ABSOLUTE: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-HOST-FILE-001"),
    "File authority root must be absolute.",
    "Resolve the authority root to an absolute application-owned path before granting access.",
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAuthorityError {
    RootNotAbsolute(PathBuf),
}

impl CodedError for FileAuthorityError {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::RootNotAbsolute(_) => &FILE_ROOT_NOT_ABSOLUTE,
        }
    }
}

impl fmt::Display for FileAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootNotAbsolute(path) => {
                write!(f, "file authority root must be absolute: {path:?}")
            }
        }
    }
}

impl Error for FileAuthorityError {}

/// Permission to observe files beneath one explicit root.
///
/// This type intentionally exposes no lexical `allows(path)` helper. Safe
/// containment depends on canonicalization and symlink-aware checks performed
/// by the concrete host implementation at the effect boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReadAuthority {
    root: PathBuf,
}

impl FileReadAuthority {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FileAuthorityError> {
        let root = root.into();
        validate_file_root(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Permission to create, replace or remove files beneath one explicit root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileWriteAuthority {
    root: PathBuf,
}

impl FileWriteAuthority {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FileAuthorityError> {
        let root = root.into();
        validate_file_root(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn validate_file_root(root: &Path) -> Result<(), FileAuthorityError> {
    if root.is_absolute() {
        Ok(())
    } else {
        Err(FileAuthorityError::RootNotAbsolute(root.to_path_buf()))
    }
}

/// Filesystem effect boundary required by managed configuration.
///
/// Mandatory-read, directory traversal, watching and metadata operations are
/// deliberately absent until a real consumer proves those semantics.
pub trait FileHost: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    fn read_optional(
        &self,
        authority: &FileReadAuthority,
        path: &Path,
    ) -> Result<Option<Vec<u8>>, Self::Error>;

    fn write(
        &self,
        authority: &FileWriteAuthority,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), Self::Error>;

    fn remove(&self, authority: &FileWriteAuthority, path: &Path) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use audiacore_errors::CodedError;

    fn absolute_root() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\audiacore-test")
        } else {
            PathBuf::from("/audiacore-test")
        }
    }

    #[test]
    fn read_and_write_authorities_keep_their_grants_distinct() {
        let root = absolute_root();
        let read = FileReadAuthority::new(root.clone()).unwrap();
        let write = FileWriteAuthority::new(root.clone()).unwrap();

        assert_eq!(read.root(), root.as_path());
        assert_eq!(write.root(), root.as_path());
    }

    #[test]
    fn relative_roots_are_rejected_with_stable_identity() {
        let error = FileReadAuthority::new("relative/root").unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-HOST-FILE-001");
    }
}
