# AudiaCore target state

Status: **TARGET CAPABILITY MAP — recovered after Stage 8**.

This document records what the finished AudiaCore/AUDiaGentic platform must be
capable of. It is intentionally separate from:

- `revalidation.md`, which records what has already been proved and accepted;
- `layer-lock.md`, which records where responsibilities may and may not live;
- `roadmap.md`, which records the order in which missing capabilities should be
  proved;
- `dependencies.md`, which records dependency admission and health decisions.

A capability may be required here without existing in production code yet.
Conversely, an accepted proof may exist only to establish a lower mechanism and
need not be the final target abstraction.

## Status vocabulary

- **PROVEN** — implemented and accepted through Stage 8 for its stated scope.
- **REQUIRED / DEFERRED** — part of the target state, but not implemented yet.
- **PARTIAL** — lower primitives exist, but the target capability does not.
- **HYPOTHESIS** — plausible future need that is not yet a committed target.
- **REJECTED** — explicitly not part of the target architecture.

## Stage 8 foundation baseline

The accepted Stage 8 foundation remains the architectural anchor. It already
proves:

- application, execution, and correlation identity with opaque
  `Application<C>` composition;
- stable coded error identity separated from human-facing presentation;
- sensitive-value handling;
- mapping-only templates;
- pure desired-versus-observed reconciliation;
- in-memory configuration resolution with ordered provenance;
- narrow file and process host contracts with explicit authority scopes;
- native file/process adapters isolated from semantic layers;
- typed event streams;
- deterministic workflow transition primitives;
- deterministic timer primitives;
- a narrow whole-file desired-state capability in `audiacore-managed-config`;
- caller-owned application-edge composition, presentation, and tracing;
- dependency and supply-chain admission rules.

These remain **PROVEN** and should not be reopened merely because higher target
capabilities were not implemented during revalidation.

## Target capability map

| Capability | Target status | Current state | Intended ownership |
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
| Managed whole-file desired state | PROVEN | `audiacore-managed-config` | lower capability/mechanism |
| Managed Content | REQUIRED / DEFERRED | requirements known; not implemented | higher capability |
| Probe / observation | REQUIRED / DEFERRED | not implemented | sibling capability |
| Software lifecycle | REQUIRED / DEFERRED | not implemented | sibling capability |
| Durable execution/orchestration | PARTIAL | workflow/events/time primitives only | application orchestration |
| Operation receipts/evidence | PARTIAL | concept locked; target use not complete | capability/application boundary |
| Artifacts / execution output | REQUIRED / DEFERRED | not implemented as target subsystem | application/runtime edge |
| Componentized application composition | REQUIRED / DEFERRED | `Application<C>` seam proven; one concrete proof only | application/bootstrap edge |
| Extension/plugin composition | REQUIRED / DEFERRED | absent | application/bootstrap edge |
| External extension sources | REQUIRED / DEFERRED | build-time Cargo sourcing only | extension/package layer |
| Extension identity + compatibility metadata | REQUIRED / DEFERRED | absent | extension/package layer |
| Extension install/update/remove lifecycle | REQUIRED / DEFERRED | absent | extension/package layer |
| Provider capability contract | REQUIRED / DEFERRED | absent | provider capability layer |
| ACP-backed provider adapters | REQUIRED / DEFERRED | absent | provider/interop layer |
| Provider capability negotiation | REQUIRED / DEFERRED | absent | provider/application boundary |
| Provider/session lifecycle | REQUIRED / DEFERRED | absent | provider/application boundary |
| MCP surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| A2A surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| ASA surface | REQUIRED / DEFERRED | absent in AudiaCore | interoperability edge |
| API / control-channel projections | REQUIRED / DEFERRED | absent in AudiaCore | application edge |
| Runtime dynamic-library loading | HYPOTHESIS | absent | extension transport only if justified |
| WASM plugin runtime | HYPOTHESIS | absent | extension transport only if justified |
| Global service locator/provider registry | REJECTED | intentionally absent | nowhere |
| Generic manager framework | REJECTED | intentionally absent | nowhere |

## Managed Config versus Managed Content

We have moved on to **Managed Content as the target capability**, but we have
**not replaced `audiacore-managed-config` in code**.

`audiacore-managed-config` is the accepted Stage 6D whole-file proof. It owns a
small, useful lower mechanism:

```text
optional whole-file observation
        + desired bytes
        -> pure plan
        -> apply through explicit file authority
```

It does not own partial content, contribution identity, structured member
ownership, prune/restore semantics, coordinated multi-resource changes, or
rollback/compensation.

Managed Content is the target higher-level capability and must support the
semantics already evidenced by AUDiaGentic recipes, including:

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
- operation rollback/compensation;
- auditable operation receipts.

The current crate may later remain as a lower whole-file mechanism, be absorbed
behind Managed Content, or be retired after an equivalent replacement is
proved. Do not decide that by renaming the crate early. Do not grow Managed
Content into `audiacore-managed-config` by accretion.

## Componentized composition target

The target application is composed from explicit components at the
application/bootstrap edge:

```text
configured sources / built-ins / external packages
                    |
                    v
        resolution + compatibility validation
                    |
                    v
          concrete implementations
                    |
                    v
          explicit application composition
                    |
                    v
              normal runtime
```

After composition, ordinary consumers use typed contracts and direct injected
collaborators. Discovery machinery is not a runtime service locator.

The following identities remain distinct:

- **Capability** — behaviour required by a caller.
- **Component** — a composable implementation unit.
- **Extension** — a distributable/discoverable contribution.
- **Package/source** — how an extension is obtained.
- **Instance** — one configured runtime use of a component.

For example:

```text
Capability:          AgentProvider
Component:           ClaudeAcpProvider
Extension:           audiagentic-provider-claude
Package/source:      external Git/package/install source
Configured instance: claude-primary
```

First-party and externally supplied implementations must use the same component
seam after resolution. Core and reusable capability crates must not depend on
extension discovery or package machinery.

## Extension source target

Extensions are not tied to a repository-local `plugins/` directory. The target
must be able to represent implementations obtained from different locations,
including:

- built-in/first-party packages;
- another Git repository;
- an installed package;
- a configured local/private location;
- other package sources justified later.

The first implementation step should prefer ordinary Rust crates and explicit
build/startup composition. Runtime-loaded `.dll`/`.so`/`.dylib`, WASM, or
out-of-process extension transports remain separate choices that require real
compatibility, security, isolation, and deployment requirements before adoption.

## Provider and interoperability target

Providers are extension-friendly capability implementations, not special
orchestration frameworks. Generic application logic must not be edited for every
provider addition.

The intended direction is:

```text
application policy / orchestration
            |
            v
   provider capability contract
            |
            v
   provider implementation/extension
            |
            v
        ACP where applicable
            |
            v
 provider/session transport
```

ACP, A2A, ASA, MCP, API, and control channels are interoperability or presentation
surfaces around one canonical application/execution model. They must not each
invent parallel workflow, task, authority, status, or persistence semantics.

## Application orchestration target

Higher application orchestration may combine:

- configuration source acquisition and semantic interpretation;
- probes/observation;
- software lifecycle;
- Managed Content;
- provider/session capabilities;
- workflows, timers, retries where explicitly required;
- verification and compensation;
- artifacts/output;
- durable status and diagnostics projections.

Capabilities do not load recipes or application definitions themselves.
Recipes/use cases belong above capabilities.

## Output and observability target

Keep these separate throughout future stages:

- domain events;
- operation receipts/evidence;
- operational tracing/logs;
- execution output/artifacts;
- public status/projections;
- explicit diagnostics.

A public status surface should remain a bounded projection of durable execution
state, not a dump of diagnostic/internal provider state.

## Non-target architecture

The target does **not** require:

- a global component/provider/plugin registry available to arbitrary runtime
  code;
- a service locator or `get<T>()` dependency container;
- a universal `Manager` abstraction;
- core knowledge of providers, recipes, plugin packages, files, or config
  sources;
- configuration that implicitly grants effect authority;
- every first-party component becoming a runtime dynamic library;
- five separate execution models for ACP/A2A/ASA/MCP/API.

The target capability map may grow as concrete requirements are recovered, but a
new target must state its owner, dependencies, exclusions, and relationship to
the accepted Stage 8 layer lock before implementation begins.
