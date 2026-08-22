//! Typed application-edge composition proof.
//!
//! This crate proves direct composition of an application identity, explicit
//! policy, explicit host authorities, configured error presentation, and
//! structured tracing. It is deliberately not a service locator, registry,
//! runtime container, configuration source, or global observability service.

use audiacore_core::{Application, ExecutionContext};
use audiacore_error_catalog::ErrorCatalogue;
use audiacore_errors::{CodedError, ErrorCode};
use audiacore_host::{FileHost, FileReadAuthority, FileWriteAuthority};
use audiacore_managed_config::{
    ManagedConfigApplyResult, ManagedConfigError, ManagedConfigTarget, apply, observe, plan,
};
use audiacore_sensitive::Sensitive;
use audiacore_template::{TemplateContext, TemplateValue};

const REDACTED: &str = "[REDACTED]";

/// Behaviour policy for one managed-configuration target.
///
/// This type is source-independent: callers may construct it from resolved
/// configuration, command-line input, tests, or any other application source.
/// It carries no file authority and therefore cannot grant effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigPolicy {
    target: ManagedConfigTarget,
    desired: Option<Vec<u8>>,
}

impl ManagedConfigPolicy {
    pub fn new(target: ManagedConfigTarget, desired: Option<Vec<u8>>) -> Self {
        Self { target, desired }
    }

    pub fn target(&self) -> &ManagedConfigTarget {
        &self.target
    }

    pub fn desired(&self) -> Option<&[u8]> {
        self.desired.as_deref()
    }
}

/// Concrete, typed composition used by the Stage 7 proof.
///
/// The host implementation, authorities, and error catalogue are caller-owned
/// values. No ambient/global lookup is performed.
pub struct ManagedConfigComposition<H> {
    host: H,
    read_authority: FileReadAuthority,
    write_authority: FileWriteAuthority,
    errors: ErrorCatalogue,
}

impl<H> ManagedConfigComposition<H> {
    pub fn new(
        host: H,
        read_authority: FileReadAuthority,
        write_authority: FileWriteAuthority,
        errors: ErrorCatalogue,
    ) -> Self {
        Self {
            host,
            read_authority,
            write_authority,
            errors,
        }
    }

    pub fn host(&self) -> &H {
        &self.host
    }

    pub fn read_authority(&self) -> &FileReadAuthority {
        &self.read_authority
    }

    pub fn write_authority(&self) -> &FileWriteAuthority {
        &self.write_authority
    }

    pub fn errors(&self) -> &ErrorCatalogue {
        &self.errors
    }
}

/// Explicit mapping-only context for externally visible error messages.
///
/// Values must already be JSON-like data. Arbitrary Rust object traversal is
/// impossible. `Sensitive<T>` values have a separate insertion path that never
/// exposes the wrapped value to the template context.
#[derive(Debug, Default, Clone)]
pub struct MessageContext {
    values: TemplateContext<String, TemplateValue>,
}

impl MessageContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_public(&mut self, key: impl Into<String>, value: TemplateValue) {
        self.values.insert(key.into(), value);
    }

    pub fn insert_sensitive<T>(&mut self, key: impl Into<String>, value: &Sensitive<T>) {
        self.values.insert(key.into(), redacted_value(value));
    }

    pub fn values(&self) -> &TemplateContext<String, TemplateValue> {
        &self.values
    }
}

/// Convert a recognized sensitive value into a safe template value without
/// reading the wrapped secret. This is also usable inside nested mapping values.
pub fn redacted_value<T>(_value: &Sensitive<T>) -> TemplateValue {
    TemplateValue::String(REDACTED.to_owned())
}

/// Externally presentable error result.
///
/// Configured definitions are preferred. If catalogue lookup or rendering
/// fails, the fallback exposes only the original stable code/category and never
/// diagnostic error text or template parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentedError {
    code: ErrorCode,
    kind: String,
    message: String,
    resolution: String,
    configured: bool,
}

impl PresentedError {
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn resolution(&self) -> &str {
        &self.resolution
    }

    pub const fn configured(&self) -> bool {
        self.configured
    }
}

pub fn present_error<E: CodedError>(
    catalogue: &ErrorCatalogue,
    error: &E,
    context: &MessageContext,
) -> PresentedError {
    let code = error.code();
    match catalogue.render(code, context.values()) {
        Ok(rendered) => PresentedError {
            code,
            kind: rendered.kind().to_owned(),
            message: rendered.message().to_owned(),
            resolution: rendered.resolution().to_owned(),
            configured: true,
        },
        Err(presentation_error) => {
            tracing::warn!(
                error_code = %code,
                presentation_error = %presentation_error,
                "configured error presentation failed"
            );
            PresentedError {
                code,
                kind: code.category().as_str().to_owned(),
                message: format!("Error {code}."),
                resolution: "Use the stable error code with diagnostic logs to identify the underlying condition."
                    .to_owned(),
                configured: false,
            }
        }
    }
}

/// Execute the managed-configuration proof through explicit policy and authority.
///
/// Structured tracing is emitted only here, at the application edge. The
/// lower managed-config, host, reconciliation, and core crates remain tracing
/// free.
pub fn execute_managed_config<H: FileHost>(
    application: &Application<ManagedConfigComposition<H>>,
    execution: &ExecutionContext,
    policy: &ManagedConfigPolicy,
) -> Result<ManagedConfigApplyResult, ManagedConfigError<H::Error>> {
    let identity = application.identity();
    let composition = application.composition();
    let span = tracing::info_span!(
        "managed_config.apply",
        application_id = %identity.application_id(),
        application_instance_id = %identity.instance_id(),
        execution_id = %execution.execution_id(),
        correlation_id = %execution.correlation_id(),
    );
    let _entered = span.enter();

    let observed = match observe(
        composition.host(),
        composition.read_authority(),
        policy.target(),
    ) {
        Ok(observed) => observed,
        Err(error) => {
            tracing::error!(error_code = %error.code(), "managed configuration observation failed");
            return Err(error);
        }
    };

    let desired = policy.desired().map(<[u8]>::to_vec);
    let planned = plan(policy.target(), &observed, &desired);
    match apply(
        composition.host(),
        composition.write_authority(),
        policy.target(),
        &planned,
    ) {
        Ok(result) => {
            tracing::info!(result = ?result, "managed configuration applied");
            Ok(result)
        }
        Err(error) => {
            tracing::error!(error_code = %error.code(), "managed configuration apply failed");
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ExampleError(ErrorCode);

    impl CodedError for ExampleError {
        fn code(&self) -> ErrorCode {
            self.0
        }
    }

    #[test]
    fn sensitive_message_values_are_redacted_without_exposure() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml(
                "test/errors.yaml",
                r#"
CON-MCONFIG-001:
  kind: constraint
  message: "Owner {owner} used token {token}."
  resolution: "Correct the ownership identity."
"#,
            )
            .unwrap();

        let secret = Sensitive::new("never-show-this".to_owned());
        let mut context = MessageContext::new();
        context.insert_public("owner", TemplateValue::String("app".to_owned()));
        context.insert_sensitive("token", &secret);

        let presented = present_error(
            &catalogue,
            &ExampleError(ErrorCode::new("CON-MCONFIG-001")),
            &context,
        );

        assert!(presented.configured());
        assert_eq!(presented.message(), "Owner app used token [REDACTED].");
        assert!(!presented.message().contains(secret.expose()));
    }

    #[test]
    fn presentation_failure_preserves_original_code_without_diagnostic_text() {
        let catalogue = ErrorCatalogue::new();
        let error = ExampleError(ErrorCode::new("IO-MCONFIG-001"));
        let presented = present_error(&catalogue, &error, &MessageContext::new());

        assert!(!presented.configured());
        assert_eq!(presented.code(), error.code());
        assert_eq!(presented.kind(), "io");
        assert_eq!(presented.message(), "Error IO-MCONFIG-001.");
        assert!(!presented.message().contains("missing"));
    }

    #[test]
    fn missing_message_parameter_falls_back_without_changing_error_identity() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml(
                "test/errors.yaml",
                r#"
CON-MCONFIG-001:
  kind: constraint
  message: "Owner {owner} is invalid."
  resolution: "Correct the owner."
"#,
            )
            .unwrap();
        let error = ExampleError(ErrorCode::new("CON-MCONFIG-001"));

        let presented = present_error(&catalogue, &error, &MessageContext::new());

        assert!(!presented.configured());
        assert_eq!(presented.code(), error.code());
        assert_eq!(presented.message(), "Error CON-MCONFIG-001.");
    }
}
