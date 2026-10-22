# Reconciliation

**Implemented:** `audiacore-reconcile`. [Source and tests](../../../crates/audiacore-reconcile/src/lib.rs)

`plan(desired, observed)` is a pure comparison over optional values `T: Clone + Eq`, returning `ReconcileAction<T>`: `Noop`, `Create(T)`, `Replace(T)`, or `Delete`.

```rust
use audiacore_reconcile::plan;
let action = plan(Some(&"new"), Some(&"old"));
```

This computes intent only; it does not decide resource ownership, grant authority, perform I/O or execute effects. [Managed Content](../managed-content/README.md) consumes this primitive.
