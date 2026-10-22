# Logging and tracing

**Implemented:** [`audiacore-logging`](../../../crates/audiacore-logging/src/lib.rs), the maintained application-edge implementation of the previously accepted [Stage 7 tracing proof](https://github.com/Audumla/AudiaCore/blob/9ee04ce0d57aee0a00707765e894f245efaf3941/crates/audiacore-application/src/lib.rs).

The core logging capability uses [Rust `tracing`](https://docs.rs/tracing/) for structured events and spans, and [`tracing-subscriber`](https://docs.rs/tracing-subscriber/) for an optional standard formatted output. It is kept in a **separately selectable crate in the AudiaCore repository**. Applications not using it do not need to depend on it.

## Application usage

```rust
use audiacore_logging::{LogLevelFilter, execution_span, log_coded_failure, stderr_subscriber};

// Application/bootstrap explicitly installs one subscriber, once.
// Library code does not initialize or replace a global subscriber.
let subscriber = stderr_subscriber(LogLevelFilter::INFO);
tracing::subscriber::set_global_default(subscriber)?;

// Scope an operation to the existing stable application and execution IDs.
let span = execution_span(app.id(), &execution_context);
let _entered = span.enter();
tracing::info!(result = "Created", "managed content applied");

// When a typed failure implements CodedError:
log_coded_failure("managed_content.apply", &failure);
```

The execution span records `application_id`, `execution_id` and `correlation_id`; a coded failure emits `error_code`, `error_category`, and an operation key. Raw `Display` / `Debug` error text is **not** automatically logged by `log_coded_failure()`. This keeps stable machine-readable errors separate from potentially sensitive diagnostics.

Use `subscriber_with_writer(level, writer)` to route formatted events to a caller-supplied writer, such as a test capture or an application-owned output handler. `stderr_subscriber()` provides the simple default. Other `tracing`-compatible subscribers or exporters can replace the standard subscriber without changing the application spans.

**Secret safety:** `Sensitive<T>` has redacted debug output when used as `?value`, but no library can prevent a caller from explicitly exposing and logging plaintext. Never pass `Sensitive::expose()` into a log event. This capability does not claim automatic arbitrary-log sanitization.

**Not included:** OpenTelemetry export, persistent log rotation, remote collection, dynamic multi-application global subscriber management, workflow/event queues. These are not prerequisites for using standard logging.

## Validation

- [Crate tests](../../../crates/audiacore-logging/src/lib.rs) exercise correlated success, coded failure, redaction, and level filtering using an isolated subscriber (no global mutation).
- [Black-box native smoke](../../../tests/smoke/observability.rs) exercises a real native Managed Content write, a denied write, and correlated logging from an application-owned subscriber.
- [Revalidation gate](../../../scripts/revalidate.sh) enforces the logging crate dependency boundary while preventing direct `tracing` dependencies from leaking into low-level AudiaCore capability crates.

**Historical decision:** Stage 7 proved `tracing` plus `tracing-subscriber`. The Stage 8 cleanup retired the proof-only application crate and, with it, the sole tracing implementation. This production logging crate restores that accepted design without reinstating the obsolete app composition.
