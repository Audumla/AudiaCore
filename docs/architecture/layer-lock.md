# AudiaCore layer lock

Status: Stage 8 architecture contract **ACCEPTED**.

This document defines enduring semantic ownership and dependency direction. It
does not define the complete product target or implementation roadmap.

Use:

- `revalidation.md` — accepted Stage 0–8 proof history/evidence;
- `layer-lock.md` — where responsibilities may and may not live;
- `target-state.md` — what the finished platform must be capable of;
- `roadmap.md` — sequencing for missing target semantics;
- `dependencies.md` — dependency admission and health decisions.

A Cargo graph can be acyclic and still be architecturally wrong when a lower
layer owns vocabulary or behaviour that belongs above it.

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

Resolved settings are data, not policy. A capability request is narrow desired
input for one capability; it is not application policy and cannot grant effects.

`audiacore-config` merges already-acquired in-memory content, resolves typed
values, and records ordered provenance. It does not discover files, read the
environment, fetch remote configuration, or acquire application policy. When a
real source-acquisition consumer appears, prefer a maintained ecosystem library
over growing source-provider infrastructure inside the resolver.

## Canonical capability/effect flow

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

Policy decides desired behaviour. Authority independently determines permitted
effects. Configuration and policy cannot mint authority.

`FileReadAuthority`, `FileWriteAuthority`, and `ProcessAuthority` are explicit
scope descriptors passed to host contracts. They make grants visible, but are
not by themselves unforgeable in-process capability objects. Filesystem
containment is enforced by the native adapter after acquiring a
`cap_std::fs::Dir`; target operations remain relative to that directory.
`ProcessAuthority` is a launch allow-list, not a descendant sandbox.

Host contracts know effect mechanics only. Native adapters know operating-system
mechanics only. Neither owns application policy, recipes, Managed Content
ownership, providers, extension packages, or application-domain concepts.

## Core and pure foundation lock

`audiacore-core` owns universal application/execution/correlation identity and
opaque caller-chosen `Application<C>` composition only. It owns no capability,
component taxonomy, policy, authority, lifecycle, provider, extension,
discovery, I/O, runtime, persistence, serialization, or observability semantics.

Pure foundation crates remain deterministic and effect-free. Importance to the
target state is not a reason to push higher vocabulary into core.

## Pure reconciliation

`audiacore-reconcile` owns only:

```text
desired + observed -> Noop | Create | Replace | Delete
```

It owns no error identity, resource/owner identifiers, authority, paths,
formats, hosts, policy, telemetry, or presentation. Resource identity and
ownership belong to the consuming capability.

## Managed Content boundary

`audiacore-managed-content` is the canonical Managed Content capability family.
The accepted Stage 8 implementation proves **only its whole-file slice**.

That current slice may:

- observe optional whole-file bytes through `FileHost`;
- compare observed bytes with desired optional bytes;
- produce an inspectable pure plan;
- apply create/replace/delete/noop through explicit file authority.

It does not yet:

- parse JSON/TOML/YAML or text sections;
- own partial content/contributions;
- prove ownership of pre-existing files;
- infer prune/restore rights;
- coordinate multiple resources;
- provide rollback/compensation semantics;
- orchestrate recipes or software lifecycle;
- watch/retry/schedule;
- provide multi-writer/CAS semantics.

`desired = None` means deletion of the entire target file. The caller must have
explicitly delegated whole-file lifecycle responsibility. File authority permits
the effect but does not prove semantic ownership of existing content.

The target terminology is deliberate: later partial/structured semantics extend
the same Managed Content capability family. Naming the Stage 8 types
`ManagedContent*` does not claim those later semantics are already implemented.

Future Managed Content requirements include whole-resource and partial
contribution ownership, structured member/collection ownership, bounded text
contributions, preservation of unrelated content, preconditions, coordinated
changes, verification, ownership-aware prune, explicit restore,
rollback/compensation, and auditable receipts/evidence.

Managed Content must not absorb configuration-source acquisition, application
policy, filesystem authority, native I/O, software lifecycle, provider/session
semantics, or recipe orchestration.

Prune, restore, and rollback remain distinct:

- prune removes a contribution established as ours from current state;
- restore deliberately reapplies an explicitly retained prior state;
- rollback compensates effects during a failed operation.

They must not collapse into a generic `undo()`.

## Sibling higher capabilities

The target map also includes Probe/Observation, Software Lifecycle, durable
application execution/orchestration, componentized application composition,
extension/package lifecycle, provider capabilities/ACP adapters, and
interoperability surfaces including MCP, A2A, ASA, API, and control channels.

These are siblings, not Managed Content subsystems, and may not be pulled into
lower layers for convenience.

## Componentized application composition lock

`Application<C>` remains opaque over caller-chosen composition. The Stage 7
`ManagedContentComposition<H>` is a proving consumer only; it is not the
canonical future component set and is not precedent for accumulating every
capability inside `audiacore-application`.

Target composition direction:

```text
configured / built-in / external implementation sources
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

Keep capability, component, extension, package/source, and configured instance
as distinct identities. First-party and externally supplied implementations use
the same capability/component seams after resolution. Reusable capabilities do
not depend on extension discovery, package resolution, or application
composition machinery.

Do not introduce service locators, dependency containers, global registries,
runtime provider registries, generic manager frameworks, implicit ambient
context, or a universal `Component` lifecycle trait before multiple real
components prove common lifecycle semantics.

## Extension/plugin boundary

External implementations from different repositories/locations are required
target capability. Extension discovery and package/source handling live at the
application/bootstrap edge.

The first extension proof should prefer ordinary Rust crates plus explicit
build/startup composition. Dynamic Rust libraries, WASM components, and
out-of-process plugin transports require separate evidence for ABI,
compatibility, deployment, isolation, and security.

Extension machinery must not remain as a globally accessible registry after
normal composition is complete.

## Provider/interoperability boundary

Providers are capability implementations, not owners of generic orchestration.
Adding a provider must not require provider-specific branches throughout generic
application logic.

ACP, A2A, ASA, MCP, API, and control channels belong at interoperability or
application edges around one canonical application/execution model. They must
not create parallel workflow, authority, persistence, status, or diagnostics
models.

## Cross-cutting outputs remain separate

Do not collapse:

- domain event — semantic fact meaningful to the application;
- operation receipt/evidence — structured result/evidence of an operation;
- operational trace/log — diagnostic/audit observation;
- execution output/artifact — information intentionally returned/retained;
- public status projection — bounded durable lifecycle/activity view;
- explicit diagnostics — bounded detail requested separately.

A log is not ownership evidence. A domain event is not a trace record. A receipt
is not presentation. Public status must not become a dump of provider or
diagnostic internals.

## Operational tracing contract

Use the Rust `tracing` ecosystem. Libraries define stable instrumentation meaning;
subscriber/exporter installation remains executable/application-edge owned.

Current canonical fields are `application_id`, `execution_id`,
`correlation_id`, `outcome`, `result` where applicable, `phase` where useful,
and stable `error_code` on coded failure/degradation. Canonical outcomes are
`success`, `failure`, and `degraded`.

ERROR means requested operation failed; WARN means degraded/fallback completion;
INFO records meaningful operation/state/effect completion; DEBUG and TRACE are
bounded diagnostics. Normal audit-level records must not broadly format
requests, configuration, contexts, credentials, secrets, or arbitrary errors.

Do not add logging/telemetry/audit manager or registry abstractions.

## Dependency health and build-versus-buy

New direct dependencies require a recorded health check. A useful API is
insufficient when maintenance, security, provenance, platform/MSRV, license,
transitive cost, or layer fit is unacceptable.

Direct third-party dependencies are approved once in root
`[workspace.dependencies]`; member crates inherit them with `workspace = true`.
Local path dependencies must resolve to declared workspace members. `deny.toml`
and SHA-pinned `cargo-deny` CI govern advisories, licenses, and sources across
the resolved graph.

If AudiaCore starts rebuilding broad source/provider discovery, package
resolution, format infrastructure, or functionality already supplied by a
healthy maintained ecosystem library, reopen build-versus-buy before adding a
custom framework.

## Semantic dependency lock

| Layer | Must not know |
| --- | --- |
| Core | files, packages, recipes, providers, extensions, config sources, logging frameworks |
| Reconcile | errors, owners, resource IDs, paths, formats, authority, hosts, policy, telemetry |
| Config resolution | application behaviour, authority, mutation, native effects, source discovery |
| Host contracts | application policy, recipes, Managed Content ownership, config sources, extensions |
| Native host | application/provider/recipe/policy/extension semantics |
| Managed Content whole-file slice | config acquisition, native host, app-domain policy, partial ownership semantics |
| Managed Content future slices | config acquisition, native I/O, application recipe/domain/provider semantics |
| Probe | application orchestration, registries/managers, native implementation details |
| Software lifecycle | app/provider concepts, native process implementation |
| Component/extension composition | capability internals, native effect mechanics, app behaviour decisions |
| Provider implementation | generic orchestration, plugin discovery, durable workflow ownership |
| Application policy | host/native/parser mechanics and effect authority |
| Presentation/status | behavioural decisions and effect execution |
| Tracing | source-of-truth domain state or ownership evidence |

A lower crate using an upper-layer term is an architecture smell even when no
Cargo dependency is present.

## Stage 8 acceptance

The original Stage 8 audit baseline was
`28ba554b1cd46bb56838ed5f9d9cc20a5881c391`, validated by run
`32624993669` (#446) on Ubuntu 24.04, macOS 15, Windows 2025 plus the
supply-chain/architecture gates.

The final Stage 8 closeout additionally aligns target-state documentation and
Managed Content terminology without broadening capability semantics. The final
closeout head/run are recorded in `revalidation.md` after validation.

The Stage 8 foundation remains deliberately small: effect-free core/foundation
semantics, explicit host ports/scopes, native effect adapters, narrow
capabilities, and an application composition/presentation/observability edge.
Future work builds upward from this lock unless a concrete required target proves
that the boundary itself is insufficient.
