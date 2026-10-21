# AudiaCore clean-room revalidation history

Status: **COMPLETE THROUGH STAGE 8**.

This file records accepted proof history only. Current ownership is in `layer-lock.md`; target capabilities are in `target-state.md`; sequencing is in `roadmap.md`.

AudiaCore rebuilt the Rust foundation from an empty repository so boundaries were earned rather than copied from prior AUDiaGentic code. Every accepted stage required explicit responsibility/exclusions, behavior and architecture gates, locked builds, and Ubuntu/macOS/Windows validation.

## Accepted invariants

- dependencies and semantic vocabulary flow downward;
- core is capability-neutral and effect-free;
- config resolution does not acquire sources or application policy;
- policy, capability requests, and effect authority are distinct;
- native effects cross narrow host contracts;
- stable error identity is separate from configured presentation;
- events, receipts/evidence, tracing, output/artifacts, status, and diagnostics are distinct;
- no service locator, global registry, generic manager, provider registry, or speculative plugin framework exists in the foundation;
- `Application<C>` is an opaque caller-chosen composition seam.

## Acceptance index

| Stage | Proof | Accepted head | Workflow |
| --- | --- | --- | --- |
| 0 | repository/build discipline | `def74266e38e69553b3481978a74d9a13ed97f57` | #2 / `32550330851` |
| 1 | core identity + opaque composition | `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126` | #16 / `32550518423` |
| 2 | stable error identity | `b87ed88b8c3d43efcb47d6564a5932505916479e` | #30 / `32550714414` |
| 3A | sensitive/template/reconcile primitives | `1278f66bf720ed3bbb5d3c05cfb77f62ea5f8d55` | #52 / `32551016872` |
| 3B | config resolution/provenance | `05031f8f5351381224d1848933ff577426ce98c7` | #72 / `32552433778` |
| 4A | file host contract + authority | `a467d1cff5ff8b5330e399c12409da08bafbab9f` | #90 / `32552758268` |
| 4B | process host contract + authority | `df7c6685b8e2048035ef70367ed7e0f9f7043ad6` | #106 / `32553116555` |
| 5A | native file effects/containment | `43453a48f5da0564f83aa56f381879f9bb710c7c` | #130 / `32553515690` |
| 5B | native process effects/lifecycle | `4baf0b3a491feb8b550f1e3ad4a82f40f1d15a16` | #154 / `32554534159` |
| 6A | events | `4fdbd7c7a0fa10e6a16af60db788f9cb3c81b088` | #178 / `32555075146` |
| 6B | workflow transitions | `8964a27ba4b7a78d047249dc10ede482cc37a561` | #198 / `32555734185` |
| 6C | time/timer primitives | `2e4c770f473ef3f5177a830590160bd9510ec8b9` | #218 / `32556194161` |
| 6D | whole-file desired-state capability | `1b99cf2d6558844b10583d130b30b9200aa41b8c` | #236 / `32556604138` |
| corrective | configured errors + template contract | `9302e83bea79a7ce82d2fd542363ac1b79ed9f97` | #302 / `32559461700` |
| 7 | application-edge composition/observability proof | `9ee04ce0d57aee0a00707765e894f245efaf3941` | #338 / `32566520037` |
| 8 audit | full layer/dependency/supply-chain audit | `28ba554b1cd46bb56838ed5f9d9cc20a5881c391` | #446 / `32624993669` |
| 8 closeout | target-state recovery + Managed Content terminology | `8e8c6974d46f1c43a9c8119583aefc07f93a7f4a` | #453 / `32706313337` |

## What remains active from those proofs

Stages 0–5 established repository discipline, core/error/config primitives, explicit host authority, and native adapters. Stages 6A–6C established reusable events/workflow/time primitives. Stage 6D established the whole-file slice now named `audiacore-managed-content`:

```text
optional observed bytes + optional desired bytes
        -> pure reconcile plan
        -> noop | create | replace | delete
        -> explicit file authority
```

Stage 7 proved that these pieces can be composed directly at an application edge, including configured error presentation, sensitive redaction, native effects, and edge-owned tracing. Its concrete `audiacore-application` crate and `revalidate-stage7.sh` were proving scaffolding, not target foundation layers; they were retired after Stage 8 and remain available through Git history.

Stage 8 established the layer/dependency baseline, recovered the target capability map, and replaced active `managed-config` terminology with Managed Content without compatibility aliases. The final implementation proof at `8e8c6974d46f1c43a9c8119583aefc07f93a7f4a` passed run #453 on Ubuntu 24.04, macOS 15, Windows 2025, and supply-chain gates.

Historical references to `managed-config` describe the original Stage 6D proof only; that terminology is not active API.

## Post-Stage-8 rule

Future work is target-capability development. A new production slice must identify its target capability, respect `layer-lock.md`, and earn only the abstractions required by its concrete proof. Lack of current application clients is not grounds to remove a valid foundation contract.
