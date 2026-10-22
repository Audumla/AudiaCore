# Managed Content

**Implemented:** `audiacore-managed-content`, whole-file desired-state management only. [Source and tests](../../../crates/audiacore-managed-content/src/lib.rs) · [cross-crate smoke test](../../../tests/smoke/foundation.rs)

```rust
use audiacore_managed_content::{observe, plan, apply};

let observed = observe(&host, &read_authority, &target)?;
let change = plan(&target, &observed, &desired_bytes);
let result = apply(&host, &write_authority, &change)?;
```

`target` is a `ManagedContentTarget`; `desired_bytes` is `Option<Vec<u8>>`. Missing observed file + desired bytes produces `Created`; changed bytes produce `Replaced`; equal bytes `Noop`; `None` desired means **delete the entire target file**.

The plan is pure. `apply()` executes through an explicitly-authorised `FileHost`; the standard implementation is `NativeFileHost`. Callers must already own lifecycle responsibility for the **whole file** before using delete/replace. File authority is not ownership evidence.

**Deferred:** structured/partial ownership, preservation of unrelated content, contributor identity, prune/restore, verification, receipts/compensation and multi-resource orchestration. See [roadmap Stage 10](../../architecture/roadmap.md).
