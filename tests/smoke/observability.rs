use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use audiacore_core::{Application, ApplicationId, CorrelationId, ExecutionContext, ExecutionId};
use audiacore_errors::CodedError;
use audiacore_host::{FileReadAuthority, FileWriteAuthority};
use audiacore_host_native::NativeFileHost;
use audiacore_logging::{
    LogLevelFilter, execution_span, log_coded_failure, subscriber_with_writer,
};
use audiacore_managed_content::{
    ManagedContentApplyResult, ManagedContentTarget, apply, observe, plan,
};

#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Result<Self, Box<dyn Error>> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "audiacore-log-smoke-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn logging_wraps_real_managed_content_effects_with_correlated_evidence()
-> Result<(), Box<dyn Error>> {
    let root = TempRoot::new()?;
    let app = Application::new(ApplicationId::new("log-smoke")?, NativeFileHost);
    let execution = ExecutionContext::new(
        ExecutionId::new("exec-log")?,
        CorrelationId::new("corr-log")?,
    );
    let reader = FileReadAuthority::new(&root.0)?;
    let writer = FileWriteAuthority::new(&root.0)?;
    let target = ManagedContentTarget::new(root.0.join("state.bin"));

    let output = Arc::new(Mutex::new(Vec::new()));
    let captured = output.clone();
    let subscriber =
        subscriber_with_writer(LogLevelFilter::INFO, move || Capture(captured.clone()));

    tracing::subscriber::with_default(subscriber, || -> Result<(), Box<dyn Error>> {
        let span = execution_span(app.id(), &execution);
        let _enter = span.enter();

        let before = observe(app.composition(), &reader, &target)?;
        let planned = plan(&target, &before, &Some(b"desired".to_vec()));
        let result = apply(app.composition(), &writer, &planned)?;
        assert_eq!(result, ManagedContentApplyResult::Created);
        tracing::info!(operation = "managed_content.apply", result = ?result, "operation completed");

        // A path outside the delegated root must fail without revealing native error text.
        let denied = ManagedContentTarget::new(root.0.join("..").join("denied.bin"));
        let impossible = plan(&denied, &None, &Some(b"secret-data".to_vec()));
        let failure = apply(app.composition(), &writer, &impossible).unwrap_err();
        assert_eq!(failure.code().as_str(), "IO-MCONTENT-001");
        log_coded_failure("managed_content.apply", &failure);
        Ok(())
    })?;

    assert_eq!(fs::read(root.0.join("state.bin"))?, b"desired");
    let log = String::from_utf8(output.lock().unwrap().clone())?;
    for needle in [
        "audiacore.execution",
        "application_id=log-smoke",
        "execution_id=exec-log",
        "correlation_id=corr-log",
        "result=Created",
        "error_code=\"IO-MCONTENT-001\"",
        "operation=\"managed_content.apply\"",
    ] {
        assert!(log.contains(needle), "missing {needle} in: {log}");
    }
    assert!(!log.contains("secret-data"));
    Ok(())
}
