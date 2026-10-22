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

**Not implemented yet:** AudiaCore logging/tracing/structured diagnostics or logger/sink initialization. Those belong in the AudiaCore application foundation but currently have no production crate. Error identity, catalogue rendering, and redaction are already available; they should be reused when logging is implemented rather than reinvented.

See [target state](../../architecture/target-state.md) and [application roadmap](../../architecture/roadmap.md).
