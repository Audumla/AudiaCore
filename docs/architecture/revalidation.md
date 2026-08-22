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
| 2 | Error contract | What stable failure identity is required without creating a global error framework? | READY |
| 3 | Pure foundation semantics | Which reusable deterministic concepts earn independent crates? | BLOCKED BY 2 |
| 4 | Host contracts | Which effects are proven strongly enough to require narrow authority-bearing contracts? | BLOCKED BY 3 |
| 5 | Native host | Can OS effects be contained behind those contracts without leaking implementation APIs? | BLOCKED BY 4 |
| 6 | Application capabilities | Which reusable behaviours belong above host/foundation and below application authority? | BLOCKED BY 5 |
| 7 | Composition + policy + observability proof | Can config-derived or programmatic policy compose capabilities while telemetry remains application-owned? | BLOCKED BY 6 |
| 8 | Full layer-lock audit | Does the complete dependency/effect graph still satisfy the original principles without exceptions? | BLOCKED BY 7 |

## Target dependency direction under test

The following is a hypothesis, not a foregone conclusion. Each stage may simplify it if the clean-room proof shows a boundary is unnecessary.

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

## Global design invariants to challenge continuously

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

### Decision

No product crate belongs in Stage 0. The stage exists only to establish a reproducible build environment, one canonical repository instruction surface, one canonical revalidation plan, and one cross-platform validation entry point.

### Evidence

- Stage implementation head: `def74266e38e69553b3481978a74d9a13ed97f57`
- Workflow run: `32550330851` (run #2)
- Ubuntu 24.04: passed
- macOS 15: passed
- Windows 2025: passed

**Stage 0 accepted.**

## Stage 1 — core

### Hypothesis

> Core should contain only semantics that every future application composition needs regardless of capability, provider, host or runtime choice.

### Accepted surface

- validated `ApplicationId`;
- validated `ExecutionId` and `CorrelationId`;
- `ExecutionContext` as an identity carrier only;
- opaque `Application<C>` composition.

### Explicitly rejected from core

- capability/component identifiers;
- lifecycle state machines;
- diagnostics/error codes;
- policy or authority;
- service registries/DI;
- I/O, environment, async runtime, tracing, serialization or provider semantics.

### Evidence

- First validation attempt failed only rustfmt; semantics were unchanged.
- Accepted head: `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126`
- Workflow run: `32550518423` (run #16)
- Ubuntu 24.04: passed
- macOS 15: passed
- Windows 2025: passed
- Core has zero crate dependencies and passes the effect/vocabulary architecture gate.

**Stage 1 accepted.**

## Stage 2 — stable error contract hypothesis

The error layer must solve only stable reusable failure identity. It must not become a global runtime error registry or force every low-level Rust error into one universal hierarchy.

Candidate minimal surface:

- `ErrorCode` with compile-time shape validation;
- `ErrorCategory` derived from the code prefix rather than independently stored;
- `ErrorDefinition { code, message, resolution }`;
- optional `CodedError` trait for reusable capability/application boundaries.

Design constraints:

- zero normal dependencies;
- one stable code identifies one semantic condition;
- canonical message/resolution are static and safe;
- dynamic context stays in the owning typed error;
- no registry, YAML loader, logger, serializer, transport envelope, manager or dependency on core;
- core remains below and independent of the error contract.

Stage 2 acceptance will require code-shape/category tests, duplicate-code enforcement in CI, zero dependencies, and no upward/application vocabulary.
