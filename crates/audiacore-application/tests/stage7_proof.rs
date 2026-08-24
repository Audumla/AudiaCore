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
    ManagedContentComposition, ManagedContentRequest, MessageContext, execute_managed_content,
    present_error,
};
use audiacore_config::{ConfigLayerId, ConfigLayers};
use audiacore_core::{Application, ApplicationId, CorrelationId, ExecutionContext, ExecutionId};
use audiacore_error_catalog::ErrorCatalogue;
use audiacore_errors::CodedError;
use audiacore_host::{FileReadAuthority, FileWriteAuthority};
use audiacore_host_native::NativeFileHost;
use audiacore_managed_content::{ManagedContentApplyResult, ManagedContentTarget};
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
fn resolved_config_builds_source_independent_request_and_native_effect_is_observable() {
    let root = TempRoot::new();
    let resolved = ConfigLayers::new()
        .merge_toml(
            ConfigLayerId::new("proof").unwrap(),
            r#"
[managed]
path = "managed.conf"
desired = "configured-value"
"#,
        )
        .unwrap()
        .resolve::<ProofConfig>()
        .unwrap();

    let target = ManagedContentTarget::new(resolved.value().managed.path.clone());
    let request_from_config = ManagedContentRequest::new(
        target.clone(),
        Some(resolved.value().managed.desired.as_bytes().to_vec()),
    );
    let direct_request = ManagedContentRequest::new(target, Some(b"configured-value".to_vec()));
    assert_eq!(request_from_config, direct_request);

    let read_authority = FileReadAuthority::new(root.path().clone()).unwrap();
    let write_authority = FileWriteAuthority::new(root.path().clone()).unwrap();
    let mut errors = ErrorCatalogue::new();
    errors
        .register_yaml(
            "audiacore-managed-content/errors.yaml",
            include_str!("../../audiacore-managed-content/errors.yaml"),
        )
        .unwrap();

    let application = Application::new(
        ApplicationId::new("stage7-app").unwrap(),
        ManagedContentComposition::new(NativeFileHost, read_authority, write_authority, errors),
    );
    let execution = execution_context();

    let captured = Arc::new(Mutex::new(Vec::new()));
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_ansi(false)
        .with_writer(move || BufferWriter(writer.clone()))
        .finish();

    let result = tracing::subscriber::with_default(subscriber, || {
        execute_managed_content(&application, &execution, &request_from_config)
    })
    .unwrap();

    assert_eq!(result, ManagedContentApplyResult::Created);
    assert_eq!(
        fs::read(root.path().join("managed.conf")).unwrap(),
        b"configured-value"
    );

    let log = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(log.contains("managed_content.apply"));
    assert!(log.contains("application_id=stage7-app"));
    assert!(log.contains("execution_id=execution-7"));
    assert!(log.contains("correlation_id=correlation-7"));
    assert!(log.contains("outcome=\"success\"") || log.contains("outcome=success"));
    assert!(log.contains("result=\"created\"") || log.contains("result=created"));
    assert!(log.contains("completed"));

    let outside_target = ManagedContentTarget::new(PathBuf::from("..").join("outside.conf"));
    let outside_request = ManagedContentRequest::new(outside_target, Some(b"denied".to_vec()));
    let error = execute_managed_content(&application, &execution, &outside_request).unwrap_err();
    assert_eq!(error.code().as_str(), "IO-MCONTENT-001");

    let presented = present_error(
        application.composition().errors(),
        &error,
        &MessageContext::new(),
    );
    assert!(presented.configured());
    assert_eq!(presented.code(), error.code());
    assert_eq!(presented.kind(), "io");
    assert_eq!(
        presented.message(),
        "Managed Content host operation failed."
    );
}
