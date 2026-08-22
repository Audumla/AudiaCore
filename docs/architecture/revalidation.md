# AudiaCore clean-room architecture revalidation

Status: **ACTIVE**

AudiaCore rebuilds the production Rust foundation from an empty repository so every layer is re-earned rather than copied. Prior AUDiaGentic work is requirements evidence only; code and boundaries are reassessed at each stage.

## Method

For every stage:

1. State responsibility and deliberate exclusions.
2. Add only the minimum code needed to prove it.
3. Add behaviour tests and architecture gates before acceptance.
4. Validate on Ubuntu, macOS, and Windows.
5. Record the accepted commit/run before building upward.
6. Reassess whether the next proposed boundary is still justified.

A green build alone is insufficient: dependency direction, effect ownership, error semantics, configuration provenance, and absence of speculative abstractions are acceptance criteria.

## Stages

| Stage | Layer / proof | Status |
| --- | --- | --- |
| 0 | Repository + build discipline | ACCEPTED |
| 1 | Core | ACCEPTED |
| 2 | Stable error contract | ACCEPTED |
| 3A | Pure deterministic primitives | ACCEPTED |
| 3B | Configuration | IN PROGRESS |
| 4 | Host contracts | BLOCKED BY 3B |
| 5 | Native host | BLOCKED BY 4 |
| 6 | Application capabilities | BLOCKED BY 5 |
| 7 | Composition + policy + observability proof | BLOCKED BY 6 |
| 8 | Full layer-lock audit | BLOCKED BY 7 |

## Dependency hypothesis under test

```text
future application/domain authority
              |
application capabilities
              |
native host implementation
              |
host contracts + authorities
              |
pure foundation semantics
              |
core
```

A stage may simplify this if a boundary cannot justify itself.

## Global design invariants

- Dependencies flow downward only.
- Core is small, capability-neutral, and effect-free.
- Pure foundation semantics are deterministic and effect-free.
- Configuration acquisition belongs at an edge; resolved typed configuration carries provenance.
- Policies are validated typed behaviour values and can be built from config or other sources.
- Authorities grant effects and are not derived implicitly from policy/configuration.
- Native effects do not leak into semantic layers.
- Stable error codes identify one semantic condition each.
- Domain evidence, operational tracing, and ordered execution output are different contracts.
- No service locator, global registry, generic manager layer, speculative provider framework, or abstraction without a consumer.

## Accepted checkpoints

### Stage 0 — repository + build discipline

Accepted head: `def74266e38e69553b3481978a74d9a13ed97f57`  
Workflow run: `32550330851` (run #2) — Ubuntu/macOS/Windows passed.

**Decision:** repository/toolchain/hygiene discipline exists before product code.

### Stage 1 — core

Accepted head: `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126`  
Workflow run: `32550518423` (run #16) — Ubuntu/macOS/Windows passed.

**Accepted:** validated application/execution/correlation identity, identity-only `ExecutionContext`, opaque `Application<C>`.

**Rejected from core:** capability/component taxonomy, lifecycle, diagnostics/error codes, policy/authority, service registries, I/O, environment, runtime, tracing, serialization and provider semantics.

### Stage 2 — stable error contract

Accepted head: `b87ed88b8c3d43efcb47d6564a5932505916479e`  
Workflow run: `32550714414` (run #30) — Ubuntu/macOS/Windows passed.

**Accepted:** zero-dependency `ErrorCode`, derived `ErrorCategory`, static `ErrorDefinition`, optional `CodedError` and repository-wide duplicate-code enforcement.

**Rejected:** registry, runtime loader, logger, serializer, transport envelope, manager, core dependency and universal base-error hierarchy.

### Stage 3A — deterministic semantic primitives

Accepted head: `1278f66bf720ed3bbb5d3c05cfb77f62ea5f8d55`  
Workflow run: `32551016872` (run #52) — Ubuntu/macOS/Windows passed.

**Accepted:**

- `audiacore-sensitive`: explicit sensitive wrapper, deterministic redaction and coded key validation;
- `audiacore-template`: tiny named-slot parsing/rendering with coded failures;
- `audiacore-reconcile`: desired/observed planning where effect intent is data.

Each crate depends only on `audiacore-errors`. None depends on core or owns filesystem/environment/process/network/runtime/telemetry behaviour.

## Stage 3B — configuration

### Required semantics

- explicit ordered in-memory TOML layers;
- deterministic recursive table merge, with later layers replacing non-table values;
- typed resolved configuration through Serde;
- deterministic provenance revision over the **exact ordered layer ids and source text**, not semantic equivalence;
- ordered layer identities retained in the result;
- stable coded failures for layer identity, parsing and typed resolution;
- no source discovery, filesystem, environment or remote acquisition;
- no policy semantics inside the configuration library.

### Dependency reassessment

The clean-room rebuild rejects the previous Figment dependency for this layer. The required contract does not need a provider framework. Direct `toml` + `serde` provides parsing and typed deserialization while AudiaCore owns the deliberately small ordered-merge/provenance semantics.

This also prevents the foundation from acquiring optional provider capabilities such as environment discovery merely because a general configuration framework exposes them.

### Planned dependency boundary

`audiacore-config` may depend on:

- `audiacore-errors`;
- `serde` for the typed deserialization contract;
- `toml` with only `std`, `parse`, and `serde` features.

It must not depend on core, host, tracing, environment/file providers or policy/capability crates.

Stage 3B is accepted only after the dependency decision, merge semantics, exact-input provenance, typed resolution and no-acquisition gates pass on all three operating systems.
