# AudiaCore roadmap

The Stage 8 foundation is closed. Future work must advance a target capability with the smallest real consumer that earns it.

## 1. Componentized application composition

Prove two independently useful components composed at the bootstrap edge.

Acceptance:
- keep `Application<C>` opaque;
- explicit typed construction/injection;
- no permanent proof-only container crate;
- no service locator, global registry, generic manager, or universal `Component` trait;
- implementation selection/lifecycle stays at bootstrap.

## 2. Managed Content higher slice

Use one real AUDiaGentic content-management requirement. Add only the ownership semantics it needs, such as one structured member or bounded text contribution.

Acceptance:
- preserve unrelated content;
- fail closed on ambiguous ownership/input;
- explicit authority remains separate from semantic ownership;
- inspectable observe -> plan -> apply -> verify flow;
- add evidence/receipts only when required by prune/restore reasoning.

## 3. Probe / observation

Introduce a sibling observation capability only for a real reusable observation need. No probe registry/provider framework.

## 4. Software lifecycle

Build install/upgrade/uninstall/version semantics above `ProcessHost` when one real integration requires them. Package-manager mechanics stay out of the host contract.

## 5. Durable execution

Compose workflow/events/time into application-level durability only as required by a real use case: durable lifecycle state, bounded status, explicit diagnostics, artifacts, timers/retries, and compensation where needed. Persistence abstractions are earned by the first storage consumer.

## 6. External extension composition

First prove a normal Rust crate from another repository/source at build/startup time.

Acceptance:
- same typed contract as a built-in implementation;
- explicit extension and compatibility identity;
- package/source identity separate from configured instance identity;
- validation before composition;
- no runtime locator after composition.

Dynamic libraries, WASM, or out-of-process plugin transports remain separate later decisions.

## 7. Provider / ACP ecosystem

From a real ACP-capable provider, establish the provider request/result boundary, one implementation, ACP session integration, then capability negotiation from differences between at least two providers. Provider implementations do not own generic orchestration or plugin discovery.

## 8. Interoperability surfaces

Add MCP, A2A, ASA, API, and control channels as adapters/projections around the canonical application/execution model. Do not create parallel task, authority, workflow, status, or persistence models.
