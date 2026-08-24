# AudiaCore clean-room architecture revalidation

Status: **COMPLETE THROUGH STAGE 8**.

This document is the accepted proof history for the clean-room Rust foundation.
It is not the future capability roadmap.

Use:

- `layer-lock.md` for enduring semantic ownership/dependency rules;
- `target-state.md` for required and deferred platform capabilities;
- `roadmap.md` for post-Stage-8 proof sequencing;
- `dependencies.md` for dependency admission/health decisions.

## Method

AudiaCore rebuilt the production Rust foundation from an empty repository so
each boundary was re-earned rather than copied from the prior AUDiaGentic code.
Prior work was treated as requirements evidence, not implementation authority.

Every accepted stage had to:

1. state responsibility and exclusions before acceptance;
2. add only enough code to prove that responsibility;
3. add behaviour tests and architecture gates;
4. pass formatting, Clippy `-D warnings`, and tests with the committed lockfile;
5. pass Ubuntu, macOS, and Windows;
6. preserve downward dependency and semantic ownership direction.

A green build alone was insufficient. Configuration provenance, policy/authority
separation, native-effect isolation, stable error identity, supply-chain policy,
and absence of speculative registries/managers/frameworks were acceptance
criteria.

## Accepted foundation model

```text
future application/domain authority
              |
application/bootstrap composition + policy + presentation/observability edge
              |
application capabilities
              |
native host implementation
              |
host contracts + explicit authorities
              |
pure foundation semantics
              |
core
```

Stage 8 validated this model for the accepted scope. Future target capabilities
build above it unless a concrete proof demonstrates that a Stage 8 boundary
itself blocks a required target.

## Global accepted invariants

- Dependencies and semantic vocabulary flow downward/inward only.
- Core remains capability-neutral and effect-free.
- Pure foundation semantics remain deterministic and effect-free.
- Configuration resolution does not acquire sources or application policy.
- Application policy expresses behaviour intent; capability requests are narrow
  inputs, not policy/configuration objects.
- Authorities grant effects independently of config/policy.
- Native effects cross narrow host contracts and remain in native adapters.
- Stable error identity is separate from configured human-facing presentation.
- Component-owned `errors.yaml` files own canonical presentation definitions.
- Domain events, operation receipts/evidence, operational tracing, execution
  output/artifacts, status projections, and diagnostics are separate concepts.
- No service locator, global registry, generic manager layer, provider registry,
  or speculative plugin framework is part of the accepted foundation.
- `Application<C>` is an opaque caller-chosen composition seam, not a canonical
  all-components container.

## Stage acceptance index

| Stage | Proof | Accepted head | Workflow | Status |
| --- | --- | --- | --- | --- |
| 0 | Repository/build discipline | `def74266e38e69553b3481978a74d9a13ed97f57` | #2 / `32550330851` | ACCEPTED |
| 1 | Core identity + opaque composition | `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126` | #16 / `32550518423` | ACCEPTED |
| 2 | Stable error identity | `b87ed88b8c3d43efcb47d6564a5932505916479e` | #30 / `32550714414` | ACCEPTED + LATER CORRECTED |
| 3A | Sensitive/template/reconcile primitives | `1278f66bf720ed3bbb5d3c05cfb77f62ea5f8d55` | #52 / `32551016872` | ACCEPTED + TEMPLATE CORRECTED |
| 3B | Configuration resolution/provenance | `05031f8f5351381224d1848933ff577426ce98c7` | #72 / `32552433778` | ACCEPTED |
| 4A | File host contract + authority | `a467d1cff5ff8b5330e399c12409da08bafbab9f` | #90 / `32552758268` | ACCEPTED |
| 4B | Process host contract + authority | `df7c6685b8e2048035ef70367ed7e0f9f7043ad6` | #106 / `32553116555` | ACCEPTED |
| 5A | Native file effects/containment | `43453a48f5da0564f83aa56f381879f9bb710c7c` | #130 / `32553515690` | ACCEPTED |
| 5B | Native process effects/lifecycle | `4baf0b3a491feb8b550f1e3ad4a82f40f1d15a16` | #154 / `32554534159` | ACCEPTED |
| 6A | Events | `4fdbd7c7a0fa10e6a16af60db788f9cb3c81b088` | #178 / `32555075146` | ACCEPTED |
| 6B | Workflow transition primitives | `8964a27ba4b7a78d047249dc10ede482cc37a561` | #198 / `32555734185` | ACCEPTED |
| 6C | Time/timer primitives | `2e4c770f473ef3f5177a830590160bd9510ec8b9` | #218 / `32556194161` | ACCEPTED |
| 6D | Managed whole-file desired-state capability | `1b99cf2d6558844b10583d130b30b9200aa41b8c` | #236 / `32556604138` | ACCEPTED |
| corrective | Configured errors + template contract | `9302e83bea79a7ce82d2fd542363ac1b79ed9f97` | #302 / `32559461700` | ACCEPTED |
| 7 | Application composition/request/observability proof | `9ee04ce0d57aee0a00707765e894f245efaf3941` | #338 / `32566520037` | ACCEPTED |
| 8 | Full layer/dependency/supply-chain audit | `28ba554b1cd46bb56838ed5f9d9cc20a5881c391` | #446 / `32624993669` | ACCEPTED |

## What each stage established

### Stage 0 — repository/build discipline

Repository controls, pinned toolchain discipline, cross-platform CI, and
architecture-gate conventions were established before product layers. Repository
guidance such as `AGENTS.md` is not a runtime/product layer.

### Stage 1 — core

Accepted application/execution/correlation identity, identity-only execution
context, and opaque `Application<C>`. Capability/component taxonomy, lifecycle,
registries, I/O, policy, authority, provider, serialization, and runtime concepts
were deliberately excluded from core.

### Stage 2 and corrective presentation work

`audiacore-errors` ultimately owns stable code/category identity only. Typed
owning-crate errors retain diagnostic context. Human-facing canonical
message/kind/resolution definitions live in component-owned `errors.yaml`
catalogues and are rendered by caller-owned presentation machinery.

### Stage 3A — pure deterministic primitives

Accepted sensitive values, mapping-only templates, and pure reconciliation.
Templates resolve explicit JSON-like mapping data only; reconciliation owns only
desired-versus-observed planning.

### Stage 3B — configuration

Accepted ordered in-memory TOML layers, recursive override, typed Serde
resolution, and exact ordered-input provenance. Source discovery, filesystem/env
acquisition, remote configuration, and application policy remain outside the
resolver.

### Stages 4A/4B — host contracts and authority

Accepted narrow file/process effect ports plus explicit authority scopes. Host
contracts do not own application policy, recipes, Managed Content ownership,
providers, or plugin/package semantics.

### Stages 5A/5B — native effects

Accepted native file/process adapters isolated from semantic layers. Filesystem
operations use capability-relative containment through `cap-std`; process
launch authority is an allow-list, not a descendant sandbox.

### Stage 6A — events

Accepted typed streams, caller-owned sequencing, explicit retention policy, and
cursor paging. No event bus, broker, transport, retry engine, persistence,
scheduler, or global publisher registry was introduced.

### Stage 6B — workflow

Accepted deterministic workflow-local transition decisions, explicit optimistic
revision, receipts, and restorable snapshots. No scheduler, retry engine,
compensation system, persistence repository, workflow manager, or runtime was
introduced.

### Stage 6C — time

Accepted caller-supplied timestamps/deadlines and deterministic timer-set
semantics. No clock provider, sleeping, scheduler, runtime, or global timer
registry was introduced.

### Stage 6D — managed whole-file desired state

Accepted the current `audiacore-managed-config` capability:

```text
optional whole-file observation
      + desired optional bytes
      -> pure reconcile plan
      -> create | replace | delete | noop
      -> apply through explicit file authority
```

This is **not Managed Content**. It does not own partial/structured content,
contribution identity, prune/restore rights, coordinated multi-resource changes,
or rollback semantics. Managed Content is now recorded as the required target
higher capability in `target-state.md`.

### Stage 7 — application-edge proof

Accepted one narrow proving composition around `ManagedConfigComposition<H>` to
demonstrate direct typed composition, source-independent capability requests,
independently supplied authority, configured error presentation, sensitive-value
redaction, native effect execution, and edge-owned structured tracing.

That concrete composition is explicitly not the canonical future application
shape and must not accumulate every future capability in
`audiacore-application`.

### Stage 8 — full layer lock

The full repository was re-audited for both Cargo direction and semantic
ownership. Dependency admission covers normal/dev/build/target-specific direct
edges, while `cargo-deny` gates advisories/licenses/sources across the transitive
graph.

Final accepted Stage 8 head:
`28ba554b1cd46bb56838ed5f9d9cc20a5881c391`.

Final validation run:
`32624993669` (#446), successful on Ubuntu 24.04, macOS 15, and Windows 2025.

No production layer inversion requiring redesign was found.

## Post-Stage-8 rule

Revalidation is complete; future work is target-capability development, not an
indefinite extension of the clean-room audit.

Before implementing a new production slice:

1. identify its entry in `target-state.md`;
2. confirm its owner against `layer-lock.md`;
3. define the minimum concrete proof and exclusions in `roadmap.md` or the stage
   plan;
4. update target status when accepted.

Do not treat absence from Stage 0–8 as evidence that a recovered target
capability was rejected. Conversely, do not introduce a target capability into a
lower layer merely because it is important to the finished platform.
