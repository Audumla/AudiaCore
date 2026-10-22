# Sensitive values

**Implemented:** `audiacore-sensitive`. [Source and tests](../../../crates/audiacore-sensitive/src/lib.rs)

Wrap values in `Sensitive<T>` to prevent accidental plaintext exposure in debug formatting. Revealing a value is an explicit call to `expose()` or `into_inner()`. `SensitiveKey` identifies validated secret keys; `redact_text()` provides deterministic string redaction against explicitly supplied sensitive values.

```rust
use audiacore_sensitive::Sensitive;

let token = Sensitive::new(String::from("private-token"));
// Token access is explicit:
let value = token.expose();
```

**Not implemented here:** a secret vault, permission system, cryptographic storage, automatic sanitization of arbitrary logs. Applications must avoid passing exposed values into diagnostics.
