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
| Componentized application composition | REQUIRED / DEFERRED | bootstrap/application edge; no current crate |
| Durable execution/orchestration | PARTIAL | workflow/events/time primitives only |
| Artifacts / execution output | REQUIRED / DEFERRED | application/runtime edge |
| Extension identity + composition | REQUIRED / DEFERRED | bootstrap/extension edge |
| External extension sources + lifecycle | REQUIRED / DEFERRED | package/source edge |
| Provider capability contract | REQUIRED / DEFERRED | provider capability layer |
| ACP-backed provider adapters + negotiation | REQUIRED / DEFERRED | provider/interop edge |
| MCP / A2A / ASA / API / control channels | REQUIRED / DEFERRED | interoperability/application edges |
| Application observability setup | REQUIRED / DEFERRED | executable/application edge |
| Runtime dynamic-library loading | HYPOTHESIS | only if deployment requirements justify it |
| WASM plugin runtime | HYPOTHESIS | only if isolation/portability requirements justify it |
| Global service locator/provider registry | REJECTED | nowhere |
| Generic manager framework | REJECTED | nowhere |

## Composition target

```text
configuration / built-ins / external packages
                -> resolution + compatibility validation
                -> concrete implementations
                -> explicit application/bootstrap composition
                -> typed runtime collaborators
```

Capability, component, extension, package/source, and configured instance remain distinct identities. First-party and external implementations use the same seam after resolution.

## Managed Content target

Future Managed Content slices may add structured members, identity-selected collection entries, bounded text contributions, preservation of unrelated content, ownership-aware prune, explicit restore, coordinated changes, verification, compensation, and operation evidence. Each slice must be independently earned; the whole-file slice remains narrow.

## Interoperability target

Providers and ACP/A2A/ASA/MCP/API/control surfaces wrap one canonical application/execution model. They must not create parallel workflow, authority, persistence, status, or diagnostics models.
