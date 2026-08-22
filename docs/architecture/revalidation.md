# AudiaCore clean-room architecture revalidation

Status: **ACTIVE**

AudiaCore rebuilds the production Rust foundation from an empty repository so every layer is re-earned rather than copied. Prior AUDiaGentic work is a requirements source only; code and boundaries are reassessed at each stage.

## Method

For every stage:

1. State the responsibility and what is deliberately excluded.
2. Add only the minimum code required to prove that responsibility.
3. Add behavioural tests and architecture gates before accepting the stage.
4. Validate on Ubuntu, macOS, and Windows.
5. Record the accepted commit/run before building the next layer.
6. Reassess whether the next proposed layer is still justified.

A green build alone is insufficient: dependency direction, effect ownership, error semantics, configuration provenance, and absence of speculative abstractions are acceptance criteria.

## Planned stages

| Stage | Layer / proof | Key question | Status |
| --- | --- | --- | --- |
| 0 | Repository + build discipline | Can the repository enforce cleanliness and repeatable cross-platform validation before code exists? | ACCEPTED |
| 1 | Core | What semantics are genuinely universal enough to sit at the dependency floor? | ACCEPTED |
| 2 | Error contract | What stable failure identity is required without creating a global error framework? | ACCEPTED |
| 3 | Pure foundation semantics | Which reusable deterministic concepts earn independent crates? | IN PROGRESS |
| 4 | Host contracts | Which effects are proven strongly enough to require narrow authority-bearing contracts? | BLOCKED BY 3 |
| 5 | Native host | Can OS effects be contained behind those contracts without leaking implementation APIs? | BLOCKED BY 4 |
| 6 | Application capabilities | Which reusable behaviours belong above host/foundation and below application authority? | BLOCKED BY 5 |
| 7 | Composition + policy + observability proof | Can config-derived or programmatic policy compose capabilities while telemetry remains application-owned? | BLOCKED BY 6 |
| 8 | Full layer-lock audit | Does the complete dependency/effect graph still satisfy the original principles without exceptions? | BLOCKED BY 7 |

## Target dependency direction under test

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

This is a hypothesis. A stage may simplify it if a boundary cannot justify itself.

## Global design invariants

- Dependencies flow downward only.
- Core is small, capability-neutral, and effect-free.
- Pure foundation semantics are deterministic and effect-free.
- Configuration acquisition belongs at an edge; resolved typed configuration carries provenance.
- Policies are validated typed behaviour values and can be built from config or other sources.
- Authorities grant effects and are not derived implicitly from policy/configuration.
- Native file/process/network effects do not leak into semantic layers.
- Stable error codes identify one semantic condition each.
- Domain evidence, operational tracing, and ordered execution output are different contracts.
- No service locator, global registry, generic manager layer, speculative provider framework, or abstraction without a consumer.

## Stage 0 — repository + build discipline

Accepted head: `def74266e38e69553b3481978a74d9a13ed97f57`  
Workflow run: `32550330851` (run #2)  
Ubuntu/macOS/Windows: passed.

**Decision:** repository/toolchain/hygiene discipline exists before product code.

## Stage 1 — core

Accepted head: `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126`  
Workflow run: `32550518423` (run #16)  
Ubuntu/macOS/Windows: passed.

**Accepted:** validated application/execution/correlation identity, identity-only `ExecutionContext`, opaque `Application<C>`.

**Rejected from core:** capability/component taxonomy, lifecycle, diagnostics/error codes, policy/authority, service registries, I/O, environment, runtime, tracing, serialization and provider semantics.

## Stage 2 — stable error contract

Accepted head: `b87ed88b8c3d43efcb47d6564a5932505916479e`  
Workflow run: `32550714414` (run #30)  
Ubuntu/macOS/Windows: passed.

**Accepted:** zero-dependency `ErrorCode`, derived `ErrorCategory`, static `ErrorDefinition`, optional `CodedError` trait and repository-wide duplicate-code enforcement.

**Rejected:** registry, YAML/runtime loader, logger, serializer, transport envelope, manager, dependency on core, and universal base error hierarchy.

## Stage 3 — pure foundation semantics

Stage 3 is split internally so abstractions are justified independently.

### Stage 3A — deterministic semantic primitives

Candidates:

- `sensitive`: explicit secret/redaction semantics and safe metadata keys;
- `template`: tiny deterministic named-slot rendering, not a general template engine;
- `reconcile`: pure desired-vs-observed planning with effects represented as data.

Constraints:

- only `audiacore-errors` may be depended on;
- no core dependency unless a concrete semantic need is demonstrated;
- no filesystem, environment, process, network, async runtime or telemetry;
- each public semantic failure has stable coded identity;
- no managers, registries or runtime ownership.

### Stage 3B — configuration

Configuration is evaluated separately because it is the first foundation candidate that may justify third-party parsing/merge machinery.

Required semantics:

- explicit ordered in-memory layers;
- typed resolved configuration;
- deterministic provenance revision over exact ordered inputs;
- layer identities retained in the result;
- no source discovery, filesystem, environment or remote acquisition;
- no policy semantics inside the configuration library.

The implementation dependency choice (Figment versus a smaller alternative) is deliberately not pre-locked; Stage 3B must justify it from the required behaviour.
