# AudiaCore target state

Status vocabulary: **PROVEN**, **REQUIRED / DEFERRED**, **PARTIAL**, **HYPOTHESIS**, **REJECTED**.

| Capability | Status | Current owner/state |
| --- | --- | --- |
| Core identity + opaque composition seam | PROVEN | `audiacore-core` |
| Stable coded errors | PROVEN | `audiacore-errors` |
| Sensitive values | PROVEN | `audiacore-sensitive` |
| Message templates + configured presentation | PROVEN | `audiacore-template`, `audiacore-error-catalog` |
| Pure reconciliation | PROVEN | `audiacore-reconcile` |
| Config resolution + provenance | PROVEN | `audiacore-config` |
| Config source acquisition | REQUIRED / DEFERRED | application/source edge |
| File/process host contracts | PROVEN | `audiacore-host` |
| Native file/process adapters | PROVEN | `audiacore-host-native` |
| Events | PROVEN | `audiacore-events` |
| Workflow transition primitives | PROVEN | `audiacore-workflow` |
| Time/timer primitives | PROVEN | `audiacore-time` |
| Managed Content whole-file slice | PROVEN | `audiacore-managed-content` |
| Managed Content partial/structured ownership | REQUIRED / DEFERRED | higher Managed Content slices |
| Managed Content receipts/prune/restore/compensation | REQUIRED / DEFERRED | higher Managed Content/application boundary |
| Probe / observation | REQUIRED / DEFERRED | sibling capability |
| Software lifecycle | REQUIRED / DEFERRED | sibling capability |
| Application assembly | REQUIRED / DEFERRED | application/bootstrap edge; no current crate |
| External component sourcing | REQUIRED / DEFERRED | application/build/package-source edge |
| Component/extension identity + compatibility | REQUIRED / DEFERRED | application/bootstrap edge; add only what real proofs require |
| Managed package lifecycle | REQUIRED / DEFERRED | later software/package edge; separate from composition |
| Durable execution/orchestration | PARTIAL | workflow/events/time primitives only |
| Artifacts / execution output | REQUIRED / DEFERRED | application/runtime edge |
| Provider capability contract | REQUIRED / DEFERRED | provider capability layer |
| ACP-backed provider adapters + negotiation | REQUIRED / DEFERRED | provider/interop edge |
| MCP / A2A / ASA / API / control channels | REQUIRED / DEFERRED | interoperability/application edges |
| Application observability setup | REQUIRED / DEFERRED | executable/application edge |
| Runtime dynamic-library loading | HYPOTHESIS | only if deployment requirements justify it |
| WASM plugin runtime | HYPOTHESIS | only if isolation/portability requirements justify it |
| Global service locator/provider registry | REJECTED | nowhere |
| Generic manager framework | REJECTED | nowhere |

## Application assembly target

```text
built-ins / local directories / external repositories or packages
                -> source/package resolution
                -> compatibility validation where required
                -> concrete typed implementations
                -> explicit application/bootstrap composition
                -> Application<C>
                -> typed runtime collaborators
```

Source location is a build/bootstrap concern, not runtime capability semantics. First-party and external implementations use the same typed contracts after resolution. Capability, component/extension, package/source, and configured instance remain distinct concepts; introduce explicit identities only when a real composition proof needs them.

Stage 9 must try standard Rust/Cargo source and dependency mechanisms before AudiaCore grows custom resolution infrastructure.

## Managed Content target

Future Managed Content slices may add structured members, identity-selected collection entries, bounded text contributions, preservation of unrelated content, ownership-aware prune, explicit restore, coordinated changes, verification, compensation, and operation evidence. Each slice must be independently earned; the whole-file slice remains narrow.

## Interoperability target

Providers and ACP/A2A/ASA/MCP/API/control surfaces wrap one canonical application/execution model. They must not create parallel workflow, authority, persistence, status, or diagnostics models.
