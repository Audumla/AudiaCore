# AudiaCore layer lock

Status: Stage 8 architecture contract **ACCEPTED**.

This document defines enduring semantic ownership and dependency direction. It
does not define the complete product target or the implementation roadmap.

Use the architecture records as follows:

- `revalidation.md` — accepted Stage 0–8 proof history and evidence;
- `layer-lock.md` — where responsibilities may and may not live;
- `target-state.md` — what the finished platform must be capable of;
- `roadmap.md` — the order in which missing target capabilities should be proved;
- `dependencies.md` — dependency admission and health decisions.

A dependency graph can be acyclic and still be architecturally wrong when a
lower layer owns vocabulary or behaviour that belongs above it.

## Governing rule

Sources provide data. Resolution produces validated settings. Application
policy expresses application intent. Capabilities receive narrow requests.
Reconciliation computes change. Authority permits effects. Host contracts
describe effects. Native adapters perform effects. Results describe what
happened. Presentation explains it. Tracing observes it.

No layer may acquire dependencies, vocabulary, source acquisition, effect
authority, or orchestration semantics owned by a layer above it.

## Canonical input and policy flow

```text
configuration source
      -> resolved settings + provenance
      -> application semantic interpretation
      -> typed application policy
      -> capability request
```

Configuration sources may include files, environment, CLI/UI input, databases,
remote configuration services, tests, or other backends. Source choice does not
become capability semantics.

Resolved settings are data, not policy. Application policy is behaviour intent,
not a configuration object. A capability request is the narrow desired input
for one capability; it is not application policy and cannot grant effects.

`audiacore-config` remains intentionally narrower than a source framework. It
merges already-acquired in-memory content, resolves typed values, and records
ordered input provenance. It must not discover files, read environment
variables, fetch remote configuration, or acquire application policy.

When a real source-acquisition consumer is introduced, prefer a maintained
ecosystem library over growing source-provider infrastructure in
`audiacore-config`.

## Canonical capability and effect flow

```text
application policy/use case
      -> capability request
      -> capability semantics
      -> pure reconciliation/planning where applicable
      -> explicit authority
      -> host contract
      -> native adapter
      -> external machine/resource
```

Policy decides what behaviour is desired. Authority independently determines
which effects are permitted. Configuration and policy cannot mint authority.

The public `FileReadAuthority`, `FileWriteAuthority`, and `ProcessAuthority`
values are explicit scope descriptors passed to host contracts. They make the
grant visible in APIs and keep policy/configuration separate from permission,
but they are not by themselves an unforgeable in-process security boundary.

For filesystem effects, containment is enforced at the native boundary after
`audiacore-host-native` acquires a `cap_std::fs::Dir`; target operations remain
relative to that directory capability. `ProcessAuthority` is an explicit launch
allow-list only and is not a sandbox for the launched process or descendants.

Host contracts know effect mechanics only. They do not know application policy,
recipes, package-manager policy, Managed Content ownership, providers, plugin
packages, or application-domain concepts. Native adapters know operating-system
mechanics only.

## Core and pure foundation lock

`audiacore-core` owns universal application/execution/correlation identity and
opaque caller-chosen `Application<C>` composition only. It owns no capability,
component taxonomy, policy, authority, lifecycle, provider, plugin, discovery,
I/O, runtime, persistence, serialization, or observability semantics.

Pure foundation crates remain deterministic and effect-free. A future target
capability does not justify pushing its vocabulary into core merely to make it
widely accessible.

## Pure reconciliation

`audiacore-reconcile` owns only generic desired-versus-observed planning:

```text
desired + observed -> Noop | Create | Replace | Delete
```

It owns no error identity, resource identifiers, owner identifiers, authority,
paths, formats, host access, application policy, telemetry, or presentation.
Resource identity and ownership are semantics of the consuming capability.

## Current managed whole-file capability

`audiacore-managed-config` is the accepted Stage 6D **whole-file desired-state
capability/proof**. It is not configuration-source infrastructure and it is not
the target Managed Content capability.

It may:

- observe optional whole-file bytes through `FileHost`;
- compare observed bytes with desired optional bytes;
- produce a pure plan;
- apply the plan through explicit file authority.

It does not:

- parse JSON/TOML/YAML or text sections;
- own partial content or contributions;
- prove ownership of pre-existing files;
- acquire policy or configuration sources;
- infer restore/prune rights;
- coordinate multi-resource changes;
- orchestrate recipes;
- manage packages;
- watch/retry/schedule;
- provide multi-writer/CAS semantics.

`desired = None` means deletion of the entire target file. The caller must have
explicitly delegated whole-file lifecycle responsibility. File authority permits
the effect but does not prove semantic ownership of existing content.

Do not rename this crate to Managed Content merely to match the target name, and
do not grow the target capability into this crate by accretion. Its eventual
retention, absorption, or retirement is decided by a real Managed Content proof.

## Target Managed Content boundary

Managed Content is the required higher-level capability for safely managing
application-owned contributions to external content. Its requirements are
tracked in `target-state.md` and include whole-resource or partial contribution
ownership, structured member/collection ownership, bounded text contributions,
preservation of unrelated user content, ownership-aware prune, explicit restore,
coordinated changes, verification, compensation, and auditable receipts.

Managed Content composes lower mechanisms. It must not absorb configuration
source acquisition, application policy, filesystem authority, native I/O,
software lifecycle, or recipe orchestration.

Prune, restore, and rollback remain distinct:

- prune removes a contribution we can establish as ours from current state;
- restore deliberately reapplies an explicitly retained prior state;
- rollback compensates effects during a failed operation.

They must not collapse into a generic `undo()`.

## Sibling higher capabilities

The target capability map currently includes sibling capabilities such as:

- Probe / observation;
- Software lifecycle;
- durable application execution/orchestration;
- componentized application composition;
- extension/plugin composition and package/source lifecycle;
- provider capabilities and ACP-backed provider adapters;
- interoperability surfaces including MCP, A2A, ASA, API, and control channels.

These are not Managed Content subsystems and may not be pulled into lower layers
for convenience. Their detailed target status belongs in `target-state.md`, not
in this lock.

## Componentized application composition lock

`Application<C>` remains opaque over caller-chosen composition. The Stage 7
`ManagedConfigComposition<H>` is a proving consumer, not the canonical future
component set and not precedent for accumulating every capability in
`audiacore-application`.

The target composition direction is:

```text
configured/built-in/external implementation sources
                    |
                    v
         resolution + validation
                    |
                    v
          concrete implementations
                    |
                    v
        explicit application/bootstrap composition
                    |
                    v
              normal runtime
```

Normal runtime consumers receive typed collaborators. They do not query a
service locator or global registry.

Keep these identities distinct:

- capability — required behaviour;
- component — composable implementation unit;
- extension — distributable/discoverable contribution;
- package/source — how an extension is obtained;
- instance — one configured runtime use.

First-party and externally supplied implementations use the same capability and
component seams after resolution. Reusable capability crates must not depend on
extension discovery, package resolution, or application composition machinery.

Do not introduce service locators, dependency containers, global registries,
runtime provider registries, generic manager frameworks, or implicit ambient
context as the system grows.

Do not introduce a universal `Component` lifecycle trait before multiple real
components prove common lifecycle semantics.

## Extension/plugin boundary

External implementations from different repositories or locations are part of
the target state, but runtime dynamic loading is not yet an accepted mechanism.
Extension discovery and package/source handling live at the
application/bootstrap edge.

The first extension proof should prefer ordinary Rust crates plus explicit
build/startup composition. `.dll`/`.so`/`.dylib`, WASM components, or
out-of-process plugin transports require separate evidence for ABI,
compatibility, deployment, isolation, and security needs.

Extension machinery must not become a globally accessible registry after normal
composition is complete.

## Provider and interoperability boundary

Provider implementations are capability implementations, not owners of generic
application orchestration. Adding a provider must not require provider-specific
branches throughout generic orchestration.

ACP, A2A, ASA, MCP, API, and control channels belong at interoperability or
application edges around one canonical application/execution model. They must
not create parallel workflow, authority, persistence, status, or diagnostics
models.

## Cross-cutting outputs remain separate

Do not collapse these into one generic event abstraction:

- domain event — semantic fact meaningful to the consuming application;
- operation receipt/evidence — structured result of a capability operation;
- operational trace/log — diagnostic/audit observation of execution;
- execution output/artifact — information intentionally returned or retained for
  a caller;
- public status projection — bounded durable lifecycle/activity projection;
- explicit diagnostics — bounded diagnostic detail requested separately.

A log is not durable ownership evidence. A domain event is not a trace record. A
receipt is not user-facing presentation. Public status must not become a dump of
provider or diagnostic internals.

## Operational tracing contract

Use the established Rust `tracing` ecosystem. AudiaCore defines meaning, not a
logging framework. Subscribers/exporters remain executable/application-edge
owned.

Current canonical operation fields are:

- `application_id`;
- `execution_id`;
- `correlation_id`;
- `outcome`;
- `result` where applicable;
- `phase` where needed to locate failure within an operation;
- stable `error_code` on coded failure/degradation.

Canonical outcomes are `success`, `failure`, and `degraded`.

Level semantics:

- ERROR — requested operation failed;
- WARN — operation or presentation completed in degraded/fallback form;
- INFO — meaningful operation/state/effect completion;
- DEBUG — controlled diagnostic detail;
- TRACE — fine-grained internal diagnostics.

Normal audit-level records must not broadly format requests, configuration,
contexts, credentials, secrets, or arbitrary error objects. Recognized sensitive
values remain redacted by construction.

Do not add `LoggingService`, `LoggerManager`, `TelemetryRegistry`,
`AuditRegistry`, or equivalent framework abstractions.

## Dependency health and build-versus-buy

New direct dependencies require a recorded health check before adoption. Existing
direct dependencies are periodically rechecked.

A dependency is unacceptable when credible evidence shows it is abandoned,
unmaintained, archived without a supported successor, security-neglected, or
otherwise stale for the required role. A useful API is insufficient.

The review considers maintenance/stewardship, issue/security responsiveness,
deprecation status, ecosystem/successor guidance, transitive exposure,
feature/dependency weight, MSRV/platform support, and license compatibility.

Direct third-party dependencies are approved once in root
`[workspace.dependencies]`; member crates inherit them with `workspace = true`.
Local path dependencies must resolve to declared workspace members. `deny.toml`
and the SHA-pinned `cargo-deny` CI action govern advisories, licenses, and
sources across the resolved graph.

If AudiaCore begins rebuilding broad source/provider discovery, package
resolution, format infrastructure, or other functionality supplied by a healthy
maintained ecosystem library, reopen build-versus-buy before adding custom
framework code.

## Semantic dependency lock

Cargo direction is necessary but insufficient. Each layer also has forbidden
knowledge.

| Layer | Must not know |
| --- | --- |
| Core | files, packages, recipes, providers, extensions, config sources, logging frameworks |
| Reconcile | errors, owners, resource IDs, paths, formats, authority, hosts, policy, telemetry |
| Config resolution | application behaviour, authority, mutation, native effects, source discovery |
| Host contracts | application policy, recipes, Managed Content ownership, config sources, plugins |
| Native host | application/provider/recipe/policy/extension semantics |
| Managed whole-file capability | config acquisition, native host, app-domain policy, partial ownership |
| Managed Content | config acquisition, native I/O, application recipe/domain/provider semantics |
| Probe | application orchestration, registries/managers, native implementation details |
| Software lifecycle | app/provider domain concepts, native process implementation |
| Component/extension composition | capability internals, native effect mechanics, application-domain behaviour decisions |
| Provider implementation | generic application orchestration, plugin discovery, durable workflow ownership |
| Application policy | host/native/parser mechanics and effect authority |
| Presentation/status | behavioural decisions and effect execution |
| Tracing | source-of-truth domain state or ownership evidence |

A lower crate using an upper-layer term is an architecture smell even when no
Cargo dependency is present.

## Stage 8 acceptance

Final accepted Stage 8 head: `28ba554b1cd46bb56838ed5f9d9cc20a5881c391`  
Workflow run: `32624993669` (#446) — Ubuntu 24.04, macOS 15, Windows 2025, and the
supply-chain/architecture gates passed.

The Stage 8 audit found no layer inversion requiring production redesign. The
accepted foundation remains deliberately small: effect-free core/foundation
semantics, explicit host ports and scopes, native effect adapters, narrow
capabilities, and an application composition/presentation/observability edge.

Future capabilities must be mapped in `target-state.md`, earn their own
boundaries, and build upward from this lock. A required target capability may
justify reopening a Stage 8 decision only when a concrete proof demonstrates
that the accepted boundary itself blocks the target; absence of a higher layer
is not such evidence.
