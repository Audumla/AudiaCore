# Application and identity

**Implemented:** `audiacore-core`. [Source](../../../crates/audiacore-core/src/lib.rs) · [tests](../../../crates/audiacore-core/src/lib.rs)

Use `ApplicationId`, `ExecutionId`, `CorrelationId`, and `ExecutionContext` for stable identity. `Application<C>` wraps **caller-owned concrete composition** without deciding how it is constructed or run.

```rust
use audiacore_core::{Application, ApplicationId};

let app = Application::new(ApplicationId::new("example")?, my_composition);
let composition = app.composition();
```

The application/bootstrap explicitly resolves implementations through Cargo, constructs them, and passes concrete types into `Application<C>`. Use `composition_mut()`, `map()`, or `into_composition()` when needed.

**Not implemented here:** service locator, DI container, component discovery, application runtime, provider/ACP behavior, lifecycle registry. See the [greenfield application checkpoint](../../architecture/roadmap.md).
