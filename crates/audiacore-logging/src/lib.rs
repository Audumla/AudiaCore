//! Structured application-edge logging and tracing for AudiaCore.
//!
//! This is the maintained implementation of the Stage 7 tracing decision.
//! AudiaCore capabilities can remain logging-agnostic while the composition
//! edge attaches stable execution identity and error codes to operations.
//!
//! Applications choose when/if to install a tracing subscriber; this library
//! never sets a global subscriber. Other subscriber implementations can be
//! used in place of the standard formatting subscriber.
//!
//! A sensitive value is NOT automatically redacted if the caller deliberately
//! extracts and logs its inner plaintext. Log only explicitly safe fields.

use audiacore_core::{ApplicationId, ExecutionContext};
use audiacore_errors::CodedError;
use tracing::{Span, error, info_span};
use tracing_subscriber::{filter::LevelFilter, fmt::MakeWriter};

/// Stable span fields for application execution, independent of the operation.
pub fn execution_span(application: &ApplicationId, execution: &ExecutionContext) -> Span {
    info_span!(
        "audiacore.execution",
        application_id = %application,
        execution_id = %execution.execution_id(),
        correlation_id = %execution.correlation_id(),
    )
}

/// Report a typed error without interpolating its raw diagnostic text.
/// The operation identifier is caller-owned and should not contain secrets.
pub fn log_coded_failure<E: CodedError + ?Sized>(operation: &'static str, source: &E) {
    error!(
        operation,
        error_code = source.code().as_str(),
        error_category = source.category().as_str(),
        "operation failed"
    );
}

/// Standard human-readable stderr subscriber. Caller chooses whether and how
/// to install it; no ambient/global initialization happens in this crate.
pub fn stderr_subscriber(max_level: LevelFilter) -> impl tracing::Subscriber + Send + Sync {
    subscriber_with_writer(max_level, std::io::stderr)
}

/// Standard formatted subscriber with an injected writer, useful for logs
/// captured by an app, tests, or a process-specific file handler.
///
/// For richer filtering, JSON output, or OpenTelemetry, applications may
/// construct their own tracing-compatible subscriber instead.
pub fn subscriber_with_writer<W>(
    max_level: LevelFilter,
    writer: W,
) -> impl tracing::Subscriber + Send + Sync
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_ansi(false)
        .with_writer(writer)
        .finish()
}

pub use tracing_subscriber::filter::LevelFilter as LogLevelFilter;

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use audiacore_core::{ApplicationId, CorrelationId, ExecutionContext, ExecutionId};
    use audiacore_errors::{CodedError, ErrorCode};
    use audiacore_sensitive::Sensitive;

    use super::*;

    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct ExampleFailure;
    impl CodedError for ExampleFailure {
        fn code(&self) -> ErrorCode {
            ErrorCode::new("IO-LOGGING-001")
        }
    }

    #[test]
    fn logs_correlated_success_and_coded_failure_without_secrets() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let sink = captured.clone();
        let subscriber = subscriber_with_writer(LevelFilter::INFO, move || Capture(sink.clone()));
        let app = ApplicationId::new("integration").unwrap();
        let execution = ExecutionContext::new(
            ExecutionId::new("execution-1").unwrap(),
            CorrelationId::new("correlation-1").unwrap(),
        );
        let secret = Sensitive::new("never-log-this-token".to_string());

        tracing::subscriber::with_default(subscriber, || {
            let span = execution_span(&app, &execution);
            let _entered = span.enter();
            tracing::info!(result = "Created", "managed content applied");
            tracing::info!(token = ?secret, "secret was redacted");
            log_coded_failure("managed_content.apply", &ExampleFailure);
        });

        let log = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
        for expected in [
            "audiacore.execution",
            "application_id=integration",
            "execution_id=execution-1",
            "correlation_id=correlation-1",
            "result=\"Created\"",
            "error_code=\"IO-LOGGING-001\"",
            "error_category=\"io\"",
            "operation=\"managed_content.apply\"",
            "[REDACTED]",
        ] {
            assert!(
                log.contains(expected),
                "missing {expected} from captured log: {log}"
            );
        }
        assert!(!log.contains("never-log-this-token"));
    }

    #[test]
    fn level_filter_omits_debug_without_dropping_errors() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let sink = captured.clone();
        let subscriber = subscriber_with_writer(LevelFilter::WARN, move || Capture(sink.clone()));
        tracing::subscriber::with_default(subscriber, || {
            tracing::debug!("should not appear");
            log_coded_failure("test.failed", &ExampleFailure);
        });

        let log = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
        assert!(!log.contains("should not appear"));
        assert!(log.contains("operation failed"));
    }
}
