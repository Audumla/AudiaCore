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
| 4A | File host contract + authority | IN PROGRESS |
| 4B | Process host contract + authority | BLOCKED BY 4A |
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

## Stage 4 — host contracts + authorities

Stage 4 is split by effect family. A single generic host abstraction is explicitly rejected.

### Stage 4A — file host contract

Immediate consumer requirement: managed configuration must be able to observe an optional file, atomically replace desired bytes through a native implementation, and remove a target. The contract should expose only the semantic operations required above the native layer.

Candidate surface:

- `FileReadAuthority { root }`;
- `FileWriteAuthority { root }`;
- `FileHost::read_optional`;
- `FileHost::write`;
- `FileHost::remove`.

Deliberate exclusions:

- no generic `read` method until a real consumer needs mandatory-read semantics;
- no lexical `allows(path)` helper because prefix checks cannot prove canonical/symlink-safe containment;
- no directory/list/watch API;
- no file-store/service/manager abstraction;
- no native `std::fs` effects in the contract crate;
- no config-derived authority and no policy semantics.

Authority values describe grants. The native implementation remains responsible for canonicalization and safe containment enforcement.

### Stage 4B — process host contract

Process execution is a known platform requirement but is revalidated separately because it has stronger lifecycle and secret-handling implications than filesystem operations.

The stage must prove whether the minimum contract still requires:

- explicit allow-listed program authority;
- secret-safe environment values;
- owned child lifecycle rather than one-shot `run()`;
- configurable stdio ownership;
- synchronous low-level process contracts that do not force an async runtime into the foundation.

Network and secret-provider host contracts remain excluded. Sensitive values may be carried into process requests, but secret retrieval is not itself a host facility until a real consumer proves that boundary.
