use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use audiacore_application::{
    ManagedConfigComposition, ManagedConfigPolicy, execute_managed_config,
};
use audiacore_config::{ConfigLayerId, ConfigLayers};
use audiacore_core::{Application, ApplicationId, CorrelationId, ExecutionContext, ExecutionId};
use audiacore_error_catalog::ErrorCatalogue;
use audiacore_host::{FileReadAuthority, FileWriteAuthority};
use audiacore_host_native::NativeFileHost;
use audiacore_managed_config::{ManagedConfigApplyResult, ManagedConfigTarget};
use audiacore_reconcile::OwnerId;
use serde::Deserialize;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "audiacore-stage7-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Deserialize)]
struct ProofConfig {
    managed: ManagedSettings,
}

#[derive(Debug, Deserialize)]
struct ManagedSettings {
    path: String,
    owner: String,
    desired: String,
}

#[derive(Clone)]
struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn execution_context() -> ExecutionContext {
    ExecutionContext::new(
        ExecutionId::new("execution-7").unwrap(),
        CorrelationId::new("correlation-7").unwrap(),
    )
}

#[test]
fn resolved_config_builds_source_independent_policy_and_native_effect_is_observable() {
    let root = TempRoot::new();
    let resolved = ConfigLayers::new()
        .merge_toml(
            ConfigLayerId::new("proof").unwrap(),
            r#"
[managed]
path = "managed.conf"
owner = "stage7-owner"
desired = "configured-value"
"#,
        )
        .unwrap()
        .resolve::<ProofConfig>()
        .unwrap();

    let owner = OwnerId::new(resolved.value().managed.owner.clone()).unwrap();
    let target = ManagedConfigTarget::new(resolved.value().managed.path.clone(), owner.clone());
    let policy_from_config = ManagedConfigPolicy::new(
        target.clone(),
        Some(resolved.value().managed.desired.as_bytes().to_vec()),
    );
    let direct_policy = ManagedConfigPolicy::new(target, Some(b"configured-value".to_vec()));
    assert_eq!(policy_from_config, direct_policy);

    let read_authority = FileReadAuthority::new(root.path().clone()).unwrap();
    let write_authority = FileWriteAuthority::new(root.path().clone()).unwrap();
    let mut errors = ErrorCatalogue::new();
    errors
        .register_yaml(
            "audiacore-managed-config/errors.yaml",
            include_str!("../../audiacore-managed-config/errors.yaml"),
        )
        .unwrap();

    let application = Application::new(
        ApplicationId::new("stage7-app").unwrap(),
        ManagedConfigComposition::new(NativeFileHost, read_authority, write_authority, errors),
    );
    let execution = execution_context();

    let captured = Arc::new(Mutex::new(Vec::new()));
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_writer(move || BufferWriter(writer.clone()))
        .finish();

    let result = tracing::subscriber::with_default(subscriber, || {
        execute_managed_config(&application, &execution, &policy_from_config)
    })
    .unwrap();

    assert_eq!(result, ManagedConfigApplyResult::Created);
    assert_eq!(
        fs::read(root.path().join("managed.conf")).unwrap(),
        b"configured-value"
    );

    let log = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(log.contains("managed_config.apply"));
    assert!(log.contains("application_id=stage7-app"));
    assert!(log.contains("execution_id=execution-7"));
    assert!(log.contains("correlation_id=correlation-7"));
    assert!(log.contains("result=Created"));
}
