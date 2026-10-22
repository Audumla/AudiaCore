# Configuration

**Implemented:** `audiacore-config`. [Source and tests](../../../crates/audiacore-config/src/lib.rs)

Resolve ordered, already-acquired TOML layers into a typed `ResolvedConfig<T>` with layer provenance and an exact-input revision. Later layers override earlier layers.

```rust
use audiacore_config::{ConfigLayerId, ConfigLayers};

let resolved = ConfigLayers::new()
    .merge_toml(ConfigLayerId::new("defaults")?, "[service]\nport = 9000")?
    .merge_toml(ConfigLayerId::new("local")?, "[service]\nport = 9001")?
    .resolve::<MySettings>()?;
```

`MySettings` is a caller-defined `serde::Deserialize` type. Inspect `resolved.value()`, `resolved.revision()`, and `resolved.layers()`.

**Not yet implemented:** reading files/environment/CLI/network, source watching, application-specific configuration policy. These are core-platform concerns to be earned at the acquisition/bootstrap edge, not already provided by this crate.
