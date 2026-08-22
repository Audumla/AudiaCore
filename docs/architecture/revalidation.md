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
| 3B | Configuration | ACCEPTED |
| 4A | File host contract + authority | ACCEPTED |
| 4B | Process host contract + authority | IN PROGRESS |
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

### Stage 3B — configuration

Accepted head: `05031f8f5351381224d1848933ff577426ce98c7`  
Workflow run: `32552433778` (run #72) — Ubuntu/macOS/Windows passed.

**Accepted:** explicit ordered in-memory TOML layers, recursive later-layer override, typed Serde resolution, exact-input provenance revision, retained ordered layer identities and stable coded parse/resolution failures.

**Dependency reassessment:** the clean-room rebuild rejects Figment. `audiacore-config` uses only `audiacore-errors`, `serde`, and `toml` with the narrow parsing/Serde surface required by the contract. Source discovery, filesystem/environment acquisition and policy semantics remain outside the crate.

**Additional correction found by revalidation:** derived `Default` would have initialized provenance differently from `ConfigLayers::new()`. `Default` now delegates to `new()` and a regression test locks that invariant.

### Stage 4A — file host contract

Accepted head: `a467d1cff5ff8b5330e399c12409da08bafbab9f`  
Workflow run: `32552758268` (run #90) — Ubuntu/macOS/Windows passed.

**Accepted:** separate read/write authority values with absolute roots and a file effect contract containing only `read_optional`, `write`, and `remove`.

**Clean-room reductions from the previous implementation:** mandatory `read` is omitted because no accepted capability consumes it; there is no lexical `allows(path)` API; relative authority roots are rejected so grants do not silently depend on process CWD.

Authority values describe grants only. Canonicalization, root existence, symlink handling and safe containment remain responsibilities of the concrete effect implementation.

## Stage 4B — process host contract

Process execution is a known platform requirement and is revalidated separately because it has stronger lifecycle and secret-handling implications than filesystem operations.

### Required semantics under test

- launch authority is an explicit allow-list of absolute program paths;
- the requested executable path is absolute so spawning never depends implicitly on process CWD;
- an optional child working directory, when supplied, is also absolute;
- process environment values are explicitly sensitive and never appear in `Debug` output;
- ambient environment inheritance is disabled by default and can only be enabled explicitly;
- stdio ownership is explicit (`pipe`, `null`, or `inherit`);
- spawning returns an owned child lifecycle rather than collapsing execution into a one-shot `run()` operation;
- the child boundary exposes stream ownership, `try_wait`, `wait`, and `kill` without forcing an async runtime into the host layer.

### Clean-room reductions to test

The previous process contract exposed both borrowed and owned stdio accessors. Stage 4B will attempt the smaller contract: **owned `take_*` stream access only**. A later runtime can retain those streams however it needs without expanding this low-level contract prematurely.

`ProcessAuthority` is launch authority only. It does not claim to sandbox the child or constrain descendant process, filesystem, network, or account authority after launch.

### Deliberate exclusions

- no Tokio or generic async host future;
- no process manager, scheduler, registry, session abstraction, or provider semantics;
- no descendant process-tree ownership claim;
- no network or secret-provider host contracts;
- no environment acquisition inside the host contract;
- no configuration or policy dependency.

Sensitive values may be carried into process requests, but secret retrieval is not itself a host facility until a real consumer proves that boundary.
