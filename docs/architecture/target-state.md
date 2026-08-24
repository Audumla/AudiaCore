# AudiaCore target state

Status: **TARGET CAPABILITY MAP — Stage 8 baseline**.

This document records what the finished AudiaCore/AUDiaGentic platform must be
capable of. It is separate from:

- `revalidation.md` — what Stage 0–8 proved and accepted;
- `layer-lock.md` — where responsibilities may and may not live;
- `roadmap.md` — the order in which missing target semantics should be proved;
- `dependencies.md` — dependency admission and health decisions.

A target capability can be only partially implemented. Target naming does not
imply that every future semantic of that capability already exists.

## Status vocabulary

- **PROVEN** — implemented and accepted for the stated scope.
- **PARTIAL** — the target capability exists, but required target semantics remain.
- **REQUIRED / DEFERRED** — committed target, not implemented yet.
- **HYPOTHESIS** — plausible future need, not yet committed.
- **REJECTED** — explicitly not part of the target architecture.

## Stage 8 foundation baseline

Stage 8 proves:

- application/execution/correlation identity with opaque `Application<C>` composition;
- stable coded error identity separated from human-facing presentation;
- sensitive-value handling and mapping-only templates;
- pure desired-versus-observed reconciliation;
- in-memory configuration resolution with ordered provenance;
- narrow file/process host contracts with explicit authority scopes;
- native file/process adapters isolated from semantic layers;
- typed event streams, workflow transitions, and deterministic timer primitives;
- the **whole-file slice of Managed Content** in `audiacore-managed-content`;
- caller-owned application-edge composition, presentation, and tracing;
- dependency and supply-chain admission rules.

These remain the architectural anchor. Missing higher target semantics do not by
themselves justify reopening the foundation.

## Target capability map

| Capability | Status | Current state | Intended ownership |
| --- | --- | --- | --- |
| Core identity + opaque composition | PROVEN | `audiacore-core` | core |
| Stable error identity | PROVEN | `audiacore-errors` | foundation |
| Sensitive values | PROVEN | `audiacore-sensitive` | foundation |
| Mapping templates | PROVEN | `audiacore-template` | foundation |
| Pure reconciliation | PROVEN | `audiacore-reconcile` | foundation |
| Config resolution + provenance | PROVEN | `audiacore-config` | foundation |
| Config source acquisition | REQUIRED / DEFERRED | intentionally absent | application/source edge |
| File/process host contracts | PROVEN | `audiacore-host` | host ports |
| Native file/process adapters | PROVEN | `audiacore-host-native` | native adapters |
| Events | PROVEN | `audiacore-events` | reusable capability |
| Workflow transition primitives | PROVEN | `audiacore-workflow` | reusable capability |
| Time/timer primitives | PROVEN | `audiacore-time` | reusable capability |
| Managed Content | PARTIAL | whole-file slice proven in `audiacore-managed-content`; structured/partial ownership deferred | application capability |
| Probe / observation | REQUIRED / DEFERRED | absent | sibling capability |
| Software lifecycle | REQUIRED / DEFERRED | absent | sibling capability |
| Durable execution/orchestration | PARTIAL | workflow/events/time primitives only | application orchestration |
| Operation receipts/evidence | PARTIAL | concept locked; target use incomplete | capability/application boundary |
| Artifacts / execution output | REQUIRED / DEFERRED | absent as target subsystem | application/runtime edge |
| Componentized application composition | REQUIRED / DEFERRED | `Application<C>` seam proven; one concrete composition proof | application/bootstrap edge |
| Extension/plugin composition | REQUIRED / DEFERRED | absent | application/bootstrap edge |
| External extension sources | REQUIRED / DEFERRED | Cargo build-time sourcing only | extension/package layer |
| Extension identity + compatibility | REQUIRED / DEFERRED | absent | extension/package layer |
| Extension install/update/remove lifecycle | REQUIRED / DEFERRED | absent | extension/package layer |
| Provider capability contract | REQUIRED / DEFERRED | absent | provider capability layer |
| ACP-backed provider adapters | REQUIRED / DEFERRED | absent | provider/interop layer |
| Provider capability negotiation | REQUIRED / DEFERRED | absent | provider/application boundary |
| Provider/session lifecycle | REQUIRED / DEFERRED | absent | provider/application boundary |
| MCP surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| A2A surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| ASA surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| API / control-channel projections | REQUIRED / DEFERRED | absent in AudiaCore | application edge |
| Runtime dynamic-library loading | HYPOTHESIS | absent | extension transport if justified |
| WASM plugin runtime | HYPOTHESIS | absent | extension transport if justified |
| Global service locator/provider registry | REJECTED | intentionally absent | nowhere |
| Generic manager framework | REJECTED | intentionally absent | nowhere |

## Managed Content target

`audiacore-managed-content` is the canonical capability family name from Stage 8
onward. Stage 8 proves only this slice:

```text
optional whole-file observation
        + desired optional bytes
        -> pure reconcile plan
        -> create | replace | delete | noop
        -> apply through explicit file authority
```

The target capability must grow from that boundary to support requirements
already evidenced by AUDiaGentic recipes:

- whole-resource ownership where explicitly delegated;
- structured object/member ownership;
- identity-selected collection entries;
- bounded text sections and anchored contributions;
- preservation of unrelated user content;
- fail-closed parsing of malformed structured content;
- version/anchor preconditions;
- coordinated multi-resource planning/application/verification;
- ownership-aware prune;
- explicit snapshot restore;
- rollback/compensation of failed operations;
- auditable operation receipts/evidence.

The Stage 8 names `ManagedContentTarget`, `ManagedContentPlan`,
`ManagedContentApplyResult`, and `ManagedContentError` describe the capability
family and its current whole-file slice. They do **not** claim that partial or
structured ownership is already implemented.

Managed Content must not absorb configuration-source acquisition, application
policy, filesystem authority, native I/O, software lifecycle, provider/session
semantics, or recipe orchestration.

## Componentized composition target

The target application is composed explicitly at bootstrap:

```text
configured / built-in / external sources
                 |
                 v
      resolution + validation
                 |
                 v
       concrete implementations
                 |
                 v
       explicit composition
                 |
                 v
           normal runtime
```

After composition, consumers use typed contracts/direct collaborators. Discovery
machinery is not a runtime service locator.

Keep these identities distinct:

- **Capability** — behaviour required by a caller.
- **Component** — composable implementation unit.
- **Extension** — distributable/discoverable contribution.
- **Package/source** — how an extension is obtained.
- **Instance** — one configured runtime use of a component.

First-party and external implementations use the same component seam after
resolution. Core and reusable capabilities do not depend on plugin discovery or
package machinery.

## Extension source target

Extensions are not tied to a repository-local `plugins/` directory. Target
sources include built-in packages, other Git repositories, installed packages,
and configured local/private locations.

The first proof should prefer ordinary Rust crates and explicit build/startup
composition. `.dll`/`.so`/`.dylib`, WASM, or out-of-process transports remain
separate hypotheses until ABI, isolation, security, and deployment requirements
justify one.

## Provider and interoperability target

Providers are extension-friendly capability implementations, not orchestration
frameworks:

```text
application policy/orchestration
          -> provider capability contract
          -> provider implementation/extension
          -> ACP where applicable
          -> provider/session transport
```

ACP, A2A, ASA, MCP, API, and control channels project one canonical
application/execution model. They must not each invent parallel workflow,
authority, persistence, status, or diagnostics semantics.

## Application orchestration target

Higher orchestration may combine configuration source acquisition, probes,
software lifecycle, Managed Content, provider/session capabilities, workflows,
timers/retries where required, verification/compensation, artifacts/output, and
durable status/diagnostics projections. Capabilities do not load recipes or
application definitions themselves.

## Output and observability target

Keep domain events, operation receipts/evidence, operational tracing, execution
output/artifacts, public status projections, and explicit diagnostics separate.
Public status is a bounded projection of durable execution state, not a dump of
provider or diagnostic internals.

## Non-target architecture

The target does **not** require a global component/provider/plugin registry,
service locator or `get<T>()` container, universal manager abstraction, core
knowledge of plugins/providers/recipes/config sources, configuration-granted
authority, every component as a dynamic library, or separate execution models
for ACP/A2A/ASA/MCP/API.

Update this map whenever a target is proved, split, superseded, or deliberately
rejected. A new target must state its owner, dependencies, exclusions, and
relationship to the Stage 8 layer lock before implementation begins.
