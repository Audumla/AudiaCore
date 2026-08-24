# AudiaCore clean-room architecture revalidation

Status: **COMPLETE THROUGH STAGE 8**.

This document is the accepted proof history for the clean-room Rust foundation.
It is not the future capability roadmap.

Use `layer-lock.md` for enduring ownership rules, `target-state.md` for target
capabilities, `roadmap.md` for future sequencing, and `dependencies.md` for
dependency decisions.

## Method

AudiaCore rebuilt the Rust foundation from an empty repository so each boundary
was re-earned rather than copied from prior AUDiaGentic code. Prior work was
requirements evidence, not implementation authority.

Every accepted stage had to define responsibility/exclusions, add only enough
code to prove them, add behaviour/architecture gates, pass formatting/Clippy/tests
with the committed lockfile, pass Ubuntu/macOS/Windows, and preserve downward
Cargo and semantic dependency direction.

A green build alone was insufficient. Configuration provenance,
policy/authority separation, native-effect isolation, stable error identity,
supply-chain policy, and absence of speculative registries/managers/frameworks
were acceptance criteria.

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

## Global accepted invariants

- Dependencies and semantic vocabulary flow downward/inward only.
- Core is capability-neutral and effect-free.
- Pure foundation semantics are deterministic/effect-free.
- Config resolution does not acquire sources or application policy.
- Application policy, capability requests, and effect authority are distinct.
- Native effects cross narrow host contracts and remain in native adapters.
- Stable error identity is separate from configured presentation.
- Domain events, receipts/evidence, tracing, output/artifacts, public status, and
  diagnostics are separate concepts.
- No service locator, global registry, generic manager, provider registry, or
  speculative plugin framework is part of the foundation.
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
| 6D | Managed Content whole-file semantics (original proof name: managed-config) | `1b99cf2d6558844b10583d130b30b9200aa41b8c` | #236 / `32556604138` | ACCEPTED |
| corrective | Configured errors + template contract | `9302e83bea79a7ce82d2fd542363ac1b79ed9f97` | #302 / `32559461700` | ACCEPTED |
| 7 | Application composition/request/observability proof | `9ee04ce0d57aee0a00707765e894f245efaf3941` | #338 / `32566520037` | ACCEPTED |
| 8 audit baseline | Full layer/dependency/supply-chain audit | `28ba554b1cd46bb56838ed5f9d9cc20a5881c391` | #446 / `32624993669` | ACCEPTED |
| 8 closeout | Target-state recovery + Managed Content terminology | `8e8c6974d46f1c43a9c8119583aefc07f93a7f4a` | #453 / `32706313337` | ACCEPTED |

## What the stages established

### Stages 0–3B

Established repository discipline, zero-dependency core identity/opaque
composition, stable coded error identity, sensitive values, mapping-only
templates, pure reconciliation, and source-free configuration
resolution/provenance. Config source acquisition and policy remained above the
resolver.

### Stages 4A–5B

Established narrow file/process ports plus explicit authority scopes and native
adapters. Filesystem effects use capability-relative `cap-std` containment.
Process authority is a launch allow-list, not a descendant sandbox.

### Stage 6A — events

Established typed streams, caller-owned sequencing, explicit retention, and
cursor paging without broker/transport/retry/persistence/scheduler semantics.

### Stage 6B — workflow

Established deterministic workflow-local transition decisions, explicit
optimistic revision, receipts, and restorable snapshots without scheduler,
retry, compensation, persistence repository, manager, or runtime ownership.

### Stage 6C — time

Established caller-supplied timestamps/deadlines and deterministic timer-set
semantics without clock acquisition, sleeping, scheduler, runtime, or global
timer registry.

### Stage 6D — Managed Content whole-file slice

The original proof was named `audiacore-managed-config`; Stage 8 closeout aligns
that implementation with the target capability name `audiacore-managed-content`
without broadening its semantics:

```text
optional whole-file observation
      + desired optional bytes
      -> pure reconcile plan
      -> create | replace | delete | noop
      -> apply through explicit file authority
```

It still does not implement partial/structured ownership, contribution identity,
prune/restore, coordinated multi-resource changes, or rollback/compensation.
Those remain future slices of the same Managed Content capability family.

### Stage 7 — application-edge proof

The proof now uses `ManagedContentRequest`, `ManagedContentComposition<H>`, and
`execute_managed_content` to demonstrate direct typed composition,
source-independent requests, separately supplied authority, configured error
presentation, sensitive redaction, native effect execution, and edge-owned
structured tracing.

That concrete composition is not the canonical future application shape and
must not accumulate every future capability in `audiacore-application`.

### Stage 8 — layer lock and target-state closeout

The full repository audit established the accepted layer/dependency/supply-chain
baseline at `28ba554b...` / run #446. The closeout recovered the explicit target
capability map and aligned active Managed Content terminology so future work does
not carry a known rename/refactor debt.

The terminology closeout removes the active `audiacore-managed-config`,
`ManagedConfig*`, `managed_config`, and `IO-MCONFIG-*` surfaces rather than
retaining compatibility aliases. Git history preserves the original proof names.

Closeout implementation head `8e8c6974d46f1c43a9c8119583aefc07f93a7f4a`
passed workflow run `32706313337` (#453): direct dependency admission,
formatting, Clippy, tests, semantic architecture gates, Stage 7 native proof on
Ubuntu 24.04/macOS 15/Windows 2025, and the supply-chain gate all passed.

## Post-Stage-8 rule

Future work is target-capability development, not indefinite clean-room audit.
Before a new production slice, identify its target entry, confirm ownership
against `layer-lock.md`, define a minimum concrete proof/exclusions, and update
target status when accepted.

Absence from Stage 0–8 does not mean a recovered target was rejected. Conversely,
importance to the target does not justify moving its vocabulary into a lower
layer.
