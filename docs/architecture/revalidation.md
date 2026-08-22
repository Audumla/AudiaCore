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
| 0 | Repository + build discipline | Can the repository enforce cleanliness and repeatable cross-platform validation before code exists? | IN PROGRESS |
| 1 | Core | What semantics are genuinely universal enough to sit at the dependency floor? | BLOCKED BY 0 |
| 2 | Error contract | What stable failure identity is required without creating a global error framework? | BLOCKED BY 1 |
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

## Stage 0 acceptance

Stage 0 contains no product crate. It is accepted when:

- the Rust toolchain is pinned;
- the repository has one canonical instruction surface and one canonical revalidation document;
- CI runs on Ubuntu, macOS, and Windows;
- the repository hygiene gate rejects legacy/runtime debris and duplicate provider instruction files;
- no generated Cargo/build output is committed;
- all three operating systems pass the same validation script.

Validation evidence: pending.
