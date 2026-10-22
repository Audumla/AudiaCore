# Templates and error presentation

**Implemented:** `audiacore-template` and `audiacore-error-catalog`. [Template source](../../../crates/audiacore-template/src/lib.rs) · [Catalogue source](../../../crates/audiacore-error-catalog/src/lib.rs)

`Template::parse()` and `Template::render()` resolve `{dotted.path}` placeholders from caller-provided JSON mappings only. They neither access Rust object properties nor read external state.

`ErrorCatalogue` accepts caller-supplied YAML definitions mapping stable error code to `message` and `resolution`. Its `register_yaml()`, `overlay_yaml()` and `render()` operations are explicit and caller-owned. Error code/category identity remains in `audiacore-errors`.

**Not implemented here:** discovery of message files, logging sinks, presentation UI, global catalogue registry. See [errors](../errors/README.md).
