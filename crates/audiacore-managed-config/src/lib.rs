//! Whole-file desired-state reconciliation over narrow file-host authority.
//!
//! This capability observes optional bytes, plans desired presence through the
//! pure reconciliation layer, and applies one resulting whole-file effect. It
//! does not parse configuration, manage partial content, prove ownership of
//! pre-existing files, watch files, retry operations, schedule work, or provide
//! multi-writer concurrency guarantees.
//!
//! `desired = None` means deletion of the entire target file. Callers must only
//! use this capability where whole-file lifecycle responsibility has already
//! been explicitly delegated. File authority permits an effect; it is not
//! semantic ownership evidence.

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use audiacore_errors::{CodedError, ErrorCode};
use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};
use audiacore_reconcile::{ReconcileAction, plan as reconcile_presence};

const HOST_OPERATION_FAILED: ErrorCode = ErrorCode::new("IO-MCONFIG-001");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigTarget {
    path: PathBuf,
}

impl ManagedConfigTarget {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigPlan {
    target: ManagedConfigTarget,
    action: ReconcileAction<Vec<u8>>,
}

impl ManagedConfigPlan {
    pub fn target(&self) -> &ManagedConfigTarget {
        &self.target
    }

    pub fn action(&self) -> &ReconcileAction<Vec<u8>> {
        &self.action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedConfigApplyResult {
    Noop,
    Created,
    Replaced,
    Deleted,
}

#[derive(Debug)]
pub enum ManagedConfigError<E> {
    Host(E),
}

impl<E> CodedError for ManagedConfigError<E> {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Host(_) => HOST_OPERATION_FAILED,
        }
    }
}

impl<E: fmt::Display> fmt::Display for ManagedConfigError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => write!(f, "managed whole-file host error: {error}"),
        }
    }
}

impl<E: Error + 'static> Error for ManagedConfigError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Host(error) => Some(error),
        }
    }
}

pub fn observe<H: FileHost>(
    host: &H,
    authority: &FileReadAuthority,
    target: &ManagedConfigTarget,
) -> Result<Option<Vec<u8>>, ManagedConfigError<H::Error>> {
    host.read_optional(authority, target.path())
        .map_err(ManagedConfigError::Host)
}

pub fn plan(
    target: &ManagedConfigTarget,
    observed: &Option<Vec<u8>>,
    desired: &Option<Vec<u8>>,
) -> ManagedConfigPlan {
    ManagedConfigPlan {
        target: target.clone(),
        action: reconcile_presence(desired.as_ref(), observed.as_ref()),
    }
}

pub fn apply<H: FileHost>(
    host: &H,
    authority: &FileWriteAuthority,
    plan: &ManagedConfigPlan,
) -> Result<ManagedConfigApplyResult, ManagedConfigError<H::Error>> {
    match plan.action() {
        ReconcileAction::Noop => Ok(ManagedConfigApplyResult::Noop),
        ReconcileAction::Create(bytes) => {
            host.write(authority, plan.target().path(), bytes)
                .map_err(ManagedConfigError::Host)?;
            Ok(ManagedConfigApplyResult::Created)
        }
        ReconcileAction::Replace(bytes) => {
            host.write(authority, plan.target().path(), bytes)
                .map_err(ManagedConfigError::Host)?;
            Ok(ManagedConfigApplyResult::Replaced)
        }
        ReconcileAction::Delete => {
            host.remove(authority, plan.target().path())
                .map_err(ManagedConfigError::Host)?;
            Ok(ManagedConfigApplyResult::Deleted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io, sync::Mutex};

    #[derive(Default)]
    struct MemoryFileHost {
        files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
    }

    impl FileHost for MemoryFileHost {
        type Error = io::Error;

        fn read_optional(
            &self,
            _authority: &FileReadAuthority,
            path: &Path,
        ) -> Result<Option<Vec<u8>>, Self::Error> {
            Ok(self.files.lock().unwrap().get(path).cloned())
        }

        fn write(
            &self,
            _authority: &FileWriteAuthority,
            path: &Path,
            bytes: &[u8],
        ) -> Result<(), Self::Error> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), bytes.to_vec());
            Ok(())
        }

        fn remove(&self, _authority: &FileWriteAuthority, path: &Path) -> Result<(), Self::Error> {
            self.files.lock().unwrap().remove(path);
            Ok(())
        }
    }

    struct FailingFileHost;

    impl FileHost for FailingFileHost {
        type Error = io::Error;

        fn read_optional(
            &self,
            _authority: &FileReadAuthority,
            _path: &Path,
        ) -> Result<Option<Vec<u8>>, Self::Error> {
            Err(io::Error::other("observe failed"))
        }

        fn write(
            &self,
            _authority: &FileWriteAuthority,
            _path: &Path,
            _bytes: &[u8],
        ) -> Result<(), Self::Error> {
            Err(io::Error::other("write failed"))
        }

        fn remove(&self, _authority: &FileWriteAuthority, _path: &Path) -> Result<(), Self::Error> {
            Err(io::Error::other("remove failed"))
        }
    }

    #[cfg(windows)]
    fn authority_root() -> PathBuf {
        PathBuf::from(r"C:\audiacore-managed-config-test")
    }

    #[cfg(not(windows))]
    fn authority_root() -> PathBuf {
        PathBuf::from("/audiacore-managed-config-test")
    }

    fn read_authority() -> FileReadAuthority {
        FileReadAuthority::new(authority_root()).unwrap()
    }

    fn write_authority() -> FileWriteAuthority {
        FileWriteAuthority::new(authority_root()).unwrap()
    }

    fn target() -> ManagedConfigTarget {
        ManagedConfigTarget::new("app.conf")
    }

    #[test]
    fn create_replace_delete_flow_is_explicit() {
        let host = MemoryFileHost::default();
        let target = target();

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let create = plan(&target, &observed, &Some(b"one".to_vec()));
        assert_eq!(create.target(), &target);
        assert_eq!(
            apply(&host, &write_authority(), &create).unwrap(),
            ManagedConfigApplyResult::Created
        );

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let replace = plan(&target, &observed, &Some(b"two".to_vec()));
        assert_eq!(
            apply(&host, &write_authority(), &replace).unwrap(),
            ManagedConfigApplyResult::Replaced
        );

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let delete = plan(&target, &observed, &None);
        assert_eq!(
            apply(&host, &write_authority(), &delete).unwrap(),
            ManagedConfigApplyResult::Deleted
        );
        assert_eq!(observe(&host, &read_authority(), &target).unwrap(), None);
    }

    #[test]
    fn unchanged_desired_bytes_produce_noop_without_host_mutation() {
        let host = MemoryFileHost::default();
        let target = target();
        host.write(&write_authority(), target.path(), b"same")
            .unwrap();

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let planned = plan(&target, &observed, &Some(b"same".to_vec()));
        assert_eq!(planned.action(), &ReconcileAction::Noop);
        assert_eq!(
            apply(&host, &write_authority(), &planned).unwrap(),
            ManagedConfigApplyResult::Noop
        );
        assert_eq!(
            observe(&host, &read_authority(), &target).unwrap(),
            observed
        );
    }

    #[test]
    fn plan_carries_the_exact_target_used_for_application() {
        let target = ManagedConfigTarget::new("one.conf");
        let planned = plan(&target, &None, &Some(b"value".to_vec()));

        assert_eq!(planned.target(), &target);
        assert_eq!(
            planned.action(),
            &ReconcileAction::Create(b"value".to_vec())
        );
    }

    #[test]
    fn observe_and_apply_host_failures_share_stable_boundary_identity() {
        let target = target();
        let observe_error = observe(&FailingFileHost, &read_authority(), &target).unwrap_err();
        assert_eq!(observe_error.code().as_str(), "IO-MCONFIG-001");
        assert_eq!(
            observe_error.source().unwrap().to_string(),
            "observe failed"
        );

        let planned = plan(&target, &None, &Some(b"value".to_vec()));
        let apply_error = apply(&FailingFileHost, &write_authority(), &planned).unwrap_err();
        assert_eq!(apply_error.code().as_str(), "IO-MCONFIG-001");
        assert_eq!(apply_error.source().unwrap().to_string(), "write failed");
    }
}
