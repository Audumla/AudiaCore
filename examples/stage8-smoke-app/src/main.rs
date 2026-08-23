use std::{error::Error, fs, io};

use audiacore_application::{
    ManagedConfigComposition, ManagedConfigRequest, execute_managed_config,
};
use audiacore_config::{ConfigLayerId, ConfigLayers};
use audiacore_core::{
    Application, ApplicationId, CorrelationId, ExecutionContext, ExecutionId,
};
use audiacore_error_catalog::ErrorCatalogue;
use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};
use audiacore_host_native::NativeFileHost;
use audiacore_managed_config::{ManagedConfigApplyResult, ManagedConfigTarget};
use serde::Deserialize;

type DynError = Box<dyn Error + Send + Sync>;

const DEMO_CONFIG: &str = r#"
target = "hello.txt"
message = "hello from AudiaCore"
"#;

#[derive(Debug, Deserialize)]
struct DemoSettings {
    target: String,
    message: String,
}

fn smoke_error(message: impl Into<String>) -> DynError {
    Box::new(io::Error::other(message.into()))
}

fn main() -> Result<(), DynError> {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .try_init()?;

    // Source acquisition belongs at the application edge. For this smoke test,
    // the source is deliberately just embedded TOML.
    let resolved = ConfigLayers::new()
        .merge_toml(ConfigLayerId::new("embedded-demo")?, DEMO_CONFIG)?
        .resolve::<DemoSettings>()?;

    let root = std::env::temp_dir().join(format!(
        "audiacore-stage8-smoke-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;

    let run = (|| -> Result<(), DynError> {
        let request = ManagedConfigRequest::new(
            ManagedConfigTarget::new(&resolved.value().target),
            Some(resolved.value().message.as_bytes().to_vec()),
        );

        let mut errors = ErrorCatalogue::new();
        errors.register_yaml(
            "audiacore-managed-config/errors.yaml",
            include_str!("../../../crates/audiacore-managed-config/errors.yaml"),
        )?;

        let application = Application::new(
            ApplicationId::new("stage8-smoke-app")?,
            ManagedConfigComposition::new(
                NativeFileHost,
                FileReadAuthority::new(&root)?,
                FileWriteAuthority::new(&root)?,
                errors,
            ),
        );
        let execution = ExecutionContext::new(
            ExecutionId::new("smoke-1")?,
            CorrelationId::new("smoke-correlation")?,
        );

        let first = execute_managed_config(&application, &execution, &request)?;
        if first != ManagedConfigApplyResult::Created {
            return Err(smoke_error(format!("expected Created, got {first:?}")));
        }

        let second = execute_managed_config(&application, &execution, &request)?;
        if second != ManagedConfigApplyResult::Noop {
            return Err(smoke_error(format!("expected Noop, got {second:?}")));
        }

        let observed = application
            .composition()
            .host()
            .read_optional(
                application.composition().read_authority(),
                request.target().path(),
            )?
            .ok_or_else(|| smoke_error("managed file was not observable after creation"))?;

        if observed != resolved.value().message.as_bytes() {
            return Err(smoke_error(
                "managed file contents did not match resolved settings",
            ));
        }

        println!(
            "AUDIACORE_SMOKE_OK first=created second=noop content={}",
            String::from_utf8_lossy(&observed)
        );
        Ok(())
    })();

    let cleanup = fs::remove_dir_all(&root);
    run?;
    cleanup?;
    Ok(())
}
