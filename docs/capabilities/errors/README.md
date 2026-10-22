# Errors, messages and templates

**Implemented:** error identity, categories, typed error integration, message templates, and caller-owned error catalogues. These are working AudiaCore facilities—not future work.

| Crate | Responsibility |
| --- | --- |
| [`audiacore-errors`](../../../crates/audiacore-errors/src/lib.rs) | Stable `ErrorCode`, prefix-derived `ErrorCategory`, `CodedError` |
| [`audiacore-template`](../../../crates/audiacore-template/src/lib.rs) | Deterministic `{dotted.path}` string rendering from supplied JSON mappings |
| [`audiacore-error-catalog`](../../../crates/audiacore-error-catalog/src/lib.rs) | YAML-backed `message`/`resolution` definitions, overlays, and human-readable error rendering |

## How to use them

An owning capability returns its normal typed Rust error and implements `CodedError` to expose stable identity and category. The application can then render a readable error using its **explicitly registered** catalogue:

```rust
use audiacore_errors::ErrorCode;
use audiacore_error_catalog::ErrorCatalogue;
use audiacore_template::{TemplateContext, TemplateValue};

let code = ErrorCode::new("VAL-SERVICE-001");
let mut catalogue = ErrorCatalogue::new();
catalogue.register_yaml("service", r#"
VAL-SERVICE-001:
  message: "Invalid field {field}."
  resolution: "Correct the supplied field."
"#)?;

let mut values = TemplateContext::new();
values.insert("field".to_string(), TemplateValue::String("port".into()));

let rendered = catalogue.render(code, &values)?;
assert_eq!(rendered.message(), "Invalid field port.");
assert_eq!(rendered.category(), "validation");
```

A catalogue rejects duplicate code registrations across owners; `overlay_yaml()` is an explicit application-level replacement operation. Messages and resolutions come from caller-provided YAML: **discovery and loading of files are not automatic**. An error's category always derives from its stable code, not from catalogue configuration.

The generic `Template::parse()`/`Template::render()` facility also works independently of error messages. This is **message formatting**, not a UI/presentation framework.

## Logging is a separate core capability

**Implemented:** application-edge structured logging and tracing via [`audiacore-logging`](../logging/README.md). The logging crate uses stable error codes and execution/correlation identity without turning error catalogues into log sinks. An application chooses and installs its subscriber; there is no global logger installed by library code. Advanced distributed telemetry/export remains deferred.

See [target state](../../architecture/target-state.md) and [application roadmap](../../architecture/roadmap.md).
