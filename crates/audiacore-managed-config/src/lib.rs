//! Managed configuration reconciliation over narrow file-host authority.
//!
//! This capability observes optional bytes, plans desired presence through the
//! pure reconciliation layer, and applies one resulting file effect. It does
//! not parse configuration, watch files, retry operations, schedule work, or
//! provide multi-writer/CAS guarantees.

use std::{error::Error, fmt, path::{Path, PathBuf}};

use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};
use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};
use audiacore_reconcile::{plan as reconcile_presence, OwnerId, ReconcileAction};

const HOST_OPERATION_FAILED: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("IO-MCONFIG-001"),
    "Managed configuration host operation failed.",
    "Inspect the underlying host error and verify the target authority and storage state.",
);
const OWNERSHIP_MISMATCH: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-MCONFIG-001"),
    "Managed configuration ownership does not match the target.",
    "Re-plan the change for the target's current ownership identity before applying it.",
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigTarget {
    path: PathBuf,
    owner: OwnerId,
}

impl ManagedConfigTarget {
    pub fn new(path: impl Into<PathBuf>, owner: OwnerId) -> Self {
        Self {
            path: path.into(),
            owner,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn owner(&self) -> &OwnerId {
        &self.owner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigPlan {
    owner: OwnerId,
    action: ReconcileAction<Vec<u8>>,
}

impl ManagedConfigPlan {
    pub fn owner(&self) -> &OwnerId {
        &self.owner
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
    OwnershipMismatch {
        expected: OwnerId,
        actual: OwnerId,
    },
}

impl<E> CodedError for ManagedConfigError<E> {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::Host(_) => &HOST_OPERATION_FAILED,
            Self::OwnershipMismatch { .. } => &OWNERSHIP_MISMATCH,
        }
    }
}

impl<E: fmt::Display> fmt::Display for ManagedConfigError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => write!(f, "managed configuration host error: {error}"),
            Self::OwnershipMismatch { expected, actual } => write!(
                f,
                "managed configuration ownership mismatch: expected {}, actual {}",
                expected.as_str(),
                actual.as_str()
            ),
        }
    }
}

impl<E: Error + 'static> Error for ManagedConfigError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Host(error) => Some(error),
            Self::OwnershipMismatch { .. } => None,
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
        owner: target.owner().clone(),
        action: reconcile_presence(desired.as_ref(), observed.as_ref()),
    }
}

pub fn apply<H: FileHost>(
    host: &H,
    authority: &FileWriteAuthority,
    target: &ManagedConfigTarget,
    plan: &ManagedConfigPlan,
) -> Result<ManagedConfigApplyResult, ManagedConfigError<H::Error>> {
    if plan.owner() != target.owner() {
        return Err(ManagedConfigError::OwnershipMismatch {
            expected: target.owner().clone(),
            actual: plan.owner().clone(),
        });
    }

    match plan.action() {
        ReconcileAction::Noop => Ok(ManagedConfigApplyResult::Noop),
        ReconcileAction::Create(bytes) => {
            host.write(authority, target.path(), bytes)
                .map_err(ManagedConfigError::Host)?;
            Ok(ManagedConfigApplyResult::Created)
        }
        ReconcileAction::Replace(bytes) => {
            host.write(authority, target.path(), bytes)
                .map_err(ManagedConfigError::Host)?;
            Ok(ManagedConfigApplyResult::Replaced)
        }
        ReconcileAction::Delete => {
            host.remove(authority, target.path())
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

        fn remove(
            &self,
            _authority: &FileWriteAuthority,
            path: &Path,
        ) -> Result<(), Self::Error> {
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

        fn remove(
            &self,
            _authority: &FileWriteAuthority,
            _path: &Path,
        ) -> Result<(), Self::Error> {
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

    fn target(owner: &str) -> ManagedConfigTarget {
        ManagedConfigTarget::new("app.conf", OwnerId::new(owner).unwrap())
    }

    #[test]
    fn create_replace_delete_flow_is_explicit() {
        let host = MemoryFileHost::default();
        let target = target("application-config");

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let create = plan(&target, &observed, &Some(b"one".to_vec()));
        assert_eq!(
            apply(&host, &write_authority(), &target, &create).unwrap(),
            ManagedConfigApplyResult::Created
        );

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let replace = plan(&target, &observed, &Some(b"two".to_vec()));
        assert_eq!(
            apply(&host, &write_authority(), &target, &replace).unwrap(),
            ManagedConfigApplyResult::Replaced
        );

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let delete = plan(&target, &observed, &None);
        assert_eq!(
            apply(&host, &write_authority(), &target, &delete).unwrap(),
            ManagedConfigApplyResult::Deleted
        );
        assert_eq!(observe(&host, &read_authority(), &target).unwrap(), None);
    }

    #[test]
    fn unchanged_desired_bytes_produce_noop_without_host_mutation() {
        let host = MemoryFileHost::default();
        let target = target("application-config");
        host.write(&write_authority(), target.path(), b"same").unwrap();

        let observed = observe(&host, &read_authority(), &target).unwrap();
        let planned = plan(&target, &observed, &Some(b"same".to_vec()));
        assert_eq!(planned.action(), &ReconcileAction::Noop);
        assert_eq!(
            apply(&host, &write_authority(), &target, &planned).unwrap(),
            ManagedConfigApplyResult::Noop
        );
        assert_eq!(observe(&host, &read_authority(), &target).unwrap(), observed);
    }

    #[test]
    fn ownership_mismatch_rejects_before_host_mutation() {
        let host = MemoryFileHost::default();
        let source = target("source-owner");
        let destination = target("destination-owner");
        let planned = plan(&source, &None, &Some(b"value".to_vec()));

        let error = apply(&host, &write_authority(), &destination, &planned).unwrap_err();

        assert_eq!(error.code().as_str(), "CON-MCONFIG-001");
        assert_eq!(observe(&host, &read_authority(), &destination).unwrap(), None);
    }

    #[test]
    fn observe_and_apply_host_failures_share_stable_boundary_identity() {
        let target = target("application-config");
        let observe_error = observe(&FailingFileHost, &read_authority(), &target).unwrap_err();
        assert_eq!(observe_error.code().as_str(), "IO-MCONFIG-001");
        assert_eq!(observe_error.source().unwrap().to_string(), "observe failed");

        let planned = plan(&target, &None, &Some(b"value".to_vec()));
        let apply_error = apply(&FailingFileHost, &write_authority(), &target, &planned).unwrap_err();
        assert_eq!(apply_error.code().as_str(), "IO-MCONFIG-001");
        assert_eq!(apply_error.source().unwrap().to_string(), "write failed");
    }
}
