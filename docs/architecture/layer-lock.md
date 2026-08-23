# AudiaCore layer lock

Status: Stage 8 architecture contract **ACCEPTED**.

This document defines semantic ownership as well as Cargo dependency direction.
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

## Input and policy side

The canonical flow is:

    configuration source
      -> resolved settings + provenance
      -> application semantic interpretation
      -> typed application policy
      -> capability request

Configuration sources can include files, environment, CLI/UI input, databases,
remote configuration services, tests, or other backends. Source choice does not
become capability semantics.

Resolved settings are data, not policy. Application policy is behaviour intent,
not a configuration object. A capability request is the narrow desired input
for one capability; it is not application policy and it cannot grant effects.

`audiacore-config` remains intentionally narrower than a source framework. It
merges already-acquired in-memory content, resolves typed values, and records
ordered input provenance. It must not discover files, read environment
variables, fetch remote configuration, or acquire application policy.

When a real source-acquisition consumer is introduced, prefer a maintained
ecosystem library over growing source-provider infrastructure in
`audiacore-config`.

## Capability and effect side

The canonical effect flow is:

    application policy/use case
      -> capability request
      -> capability semantics
      -> pure reconciliation/planning where applicable
      -> explicit authority
      -> host contract
      -> native adapter
      -> external machine/resource

Policy decides what behaviour is desired. Authority independently determines
which effects are permitted. Configuration and policy cannot mint authority.

The public `FileReadAuthority`, `FileWriteAuthority`, and `ProcessAuthority`
values are explicit scope descriptors passed to host contracts. They make the
grant visible in APIs and keep policy/configuration separate from permission,
but they are not by themselves an unforgeable in-process security boundary.
For filesystem effects, containment is enforced at the native boundary after
`audiacore-host-native` acquires a `cap_std::fs::Dir`; all target operations are
then relative to that directory capability. `ProcessAuthority` is an explicit
launch allow-list only and is not a sandbox for the launched process or its
descendants.

Host contracts know effect mechanics only. They do not know application policy,
recipes, package-manager policy, managed-content ownership, or provider/domain
concepts. Native adapters know operating-system mechanics only.

## Pure reconciliation

`audiacore-reconcile` owns only generic desired-versus-observed planning:

    desired + observed -> Noop | Create | Replace | Delete

It owns no resource identifiers, owner identifiers, authority, path semantics,
host access, application policy, telemetry, error identity, or error
presentation. Resource identity and ownership are semantics of the capability
that uses reconciliation, not of reconciliation itself.

## Current whole-file capability

`audiacore-managed-config` is currently a whole-file desired-state capability,
not the future Managed Content capability and not configuration-source
infrastructure.

It may:

- observe optional whole-file bytes through `FileHost`;
- compare observed bytes with desired optional bytes;
- produce a pure plan;
- apply the plan through explicit file authority.

It does not:

- parse JSON/TOML/YAML or text sections;
- own partial content;
- prove ownership of pre-existing files;
- acquire policy or configuration sources;
- infer restore/prune rights;
- orchestrate recipes;
- manage packages;
- watch/retry/schedule;
- provide multi-writer/CAS semantics.

`desired = None` means deletion of the entire target file. The caller must have
explicitly delegated whole-file lifecycle responsibility. File authority permits
the effect but does not prove semantic ownership of the existing content.

The Stage 7 request type is a capability request, not application policy.

## Future higher-level Managed Content capability

AUDiaGentic recipes provide concrete requirements evidence for a later,
higher-level capability that safely manages contributions to external content.

Known required semantics include:

- whole-resource ownership where explicitly delegated;
- structured object/member ownership;
- identity-selected collection entries;
- bounded text sections;
- exact/anchored line contributions;
- parser-safe preservation of unrelated user content;
- fail-closed handling of malformed structured content;
- version/anchor preconditions for patch-like changes;
- coordinated multi-resource changes;
- observation, inspectable planning, application, verification;
- ownership-aware prune;
- explicit snapshot restore;
- rollback/compensation of a failed multi-step operation;
- auditable operation receipts.

Managed Content must compose lower mechanisms. It must not absorb configuration
source acquisition, application policy, filesystem authority, native I/O, or
recipe orchestration.

Prune, restore, and rollback are distinct:

- prune removes the contribution we can establish as ours from current state;
- restore applies an explicitly retained prior state under a deliberate restore
  operation;
- rollback compensates effects during a failed operation.

They must not collapse into a generic `undo()`.

## Sibling future capabilities

Requirements evidence also supports sibling capabilities, not Managed Content
subsystems:

### Probe / observation

Likely observations include file/directory presence, executable discovery and
version, package/version state, command outcome, structured content state, text
markers, and service/endpoint reachability.

Probe semantics should remain reusable and narrow. Do not create a probe
registry or manager.

### Software lifecycle

Likely semantics include applicability, installed/version state, install,
upgrade, uninstall, prerequisite satisfaction, and verification.

Package-manager mechanics belong above `ProcessHost`, not inside it. Shared
dependency ownership must remain distinguishable from one application's
integration state. Do not add a package solver or universal resource DAG until
multiple real consumers prove the need.

### Recipe/use-case orchestration

Recipes belong above capabilities. They can combine probes, software lifecycle,
managed content, verification, and compensation. Capabilities must never load
or understand application recipe definitions.

## Cross-cutting outputs are separate

Do not collapse these into one generic event abstraction:

- domain event: semantic fact meaningful to the consuming application;
- operation receipt: structured result/evidence of a capability operation;
- operational trace/log: diagnostic/audit observation of execution;
- execution output: information intentionally returned or streamed to a caller.

A log is not durable ownership evidence. A domain event is not a trace record.
A receipt is not user-facing presentation.

## Operational tracing contract

Use the established Rust `tracing` ecosystem. AudiaCore defines meaning, not a
logging framework. Subscribers/exporters remain executable/application-edge
owned.

Normal operation records use stable structured fields rather than
component-specific prose. Current canonical fields are:

- `application_id`
- `execution_id`
- `correlation_id`
- `outcome`
- `result` where applicable
- `phase` where needed to locate failure within one operation
- stable `error_code` on coded failure/degradation

Canonical outcomes are `success`, `failure`, and `degraded`.

Level semantics:

- ERROR: the requested operation failed;
- WARN: the operation or presentation completed in degraded/fallback form;
- INFO: meaningful operation/state/effect completion;
- DEBUG: controlled diagnostic detail;
- TRACE: fine-grained internal diagnostics.

Normal audit-level records must not broadly format requests, configuration,
contexts, credentials, secrets, or arbitrary error objects. Recognized
sensitive values remain redacted by construction.

Do not add `LoggingService`, `LoggerManager`, `TelemetryRegistry`,
`AuditRegistry`, or equivalent framework abstractions.

## Dependency health and build-versus-buy

New direct dependencies require a recorded health check before adoption.
Existing direct dependencies are periodically rechecked.

A dependency is unacceptable when there is credible evidence it is abandoned,
unmaintained, archived without a supported successor, security-neglected, or
otherwise stale for the role AudiaCore requires. A useful API is not sufficient.

The review should consider:

- recent repository activity and release cadence appropriate to maturity;
- active maintainer or organization stewardship;
- issue/PR/security responsiveness;
- explicit deprecation/archive/unmaintained notices;
- current ecosystem usage and successor guidance;
- transitive dependency/security exposure;
- feature/dependency weight relative to the capability used;
- MSRV/platform compatibility;
- license compatibility.

Mature low-churn libraries do not require constant releases, but an unexplained
multi-year maintenance gap is a rejection signal for a new foundational
dependency.

### Configuration library decision, 2026-08-23

Figment was reconsidered because its provider/layer/extraction model overlaps
with configuration requirements. It is not adopted: the repository's latest
push was 2024-09-13, which fails the current maintenance gate for a new
foundation dependency.

`rust-cli/config-rs` is the preferred live candidate when a real application
needs file/environment/other source acquisition. Its repository was actively
updated in August 2026 and it is organization-maintained. It is not added to
`audiacore-config` now because source acquisition is intentionally outside that
crate and no current production consumer requires a source framework.

The current small in-memory resolver remains justified only while it stays
narrow. If AudiaCore starts rebuilding provider discovery, file/env loading,
profiles, broad format-provider infrastructure, or remote source acquisition,
the build-versus-buy decision must be reopened before adding that code.

### Current direct dependency health, 2026-08-23

The direct ecosystem dependencies currently used by the workspace were checked
during this audit:

- Serde / serde_json: actively maintained under `serde-rs`;
- `toml`: actively maintained under `toml-rs`;
- `tracing` / `tracing-subscriber`: actively maintained under `tokio-rs`;
- `yaml_serde`: actively maintained by the YAML organization as the supported
  successor/fork of the unmaintained `serde_yaml`;
- `cap-std`: actively maintained in the Bytecode Alliance capability-oriented
  Rust ecosystem, supports the required Linux/macOS/Windows families, and is
  restricted to the native-host adapter where its handle-relative filesystem
  model is required.

No known direct dependency is intentionally retained after being identified as
dead or superseded.

## Dependency admission and transitive governance

Direct third-party dependencies are approved once in root
`[workspace.dependencies]`; member crates inherit them with `workspace = true`.
The parser-based admission check covers normal, dev, build, and target-specific
dependency tables. Local path dependencies must resolve to declared workspace
members. This makes a dependency addition an explicit architectural review
rather than an incidental member-manifest edit.

The committed `deny.toml` and SHA-pinned `cargo-deny` CI action enforce the
transitive graph independently of the direct-admission rule. The accepted gate
checks known advisories, permits only the reviewed license set, rejects unknown
registries, and rejects git dependencies unless explicitly approved.

## Semantic dependency lock

Cargo direction is necessary but insufficient. Each layer also has forbidden
knowledge.

| Layer | Must not know |
| --- | --- |
| Core | files, packages, recipes, providers, config sources, logging frameworks |
| Reconcile | errors, owners, resource IDs, paths, formats, authority, hosts, policy, telemetry |
| Config resolution | application behaviour, authority, mutation, native effects |
| Host contracts | application policy, recipes, managed-content ownership, config sources |
| Native host | application/provider/recipe/policy semantics |
| Managed whole-file capability | config acquisition, native host, app-domain policy, partial ownership |
| Future Managed Content | config acquisition, native I/O, application recipe/domain semantics |
| Software lifecycle | app/provider domain concepts, native process implementation |
| Application policy | host/native/parser mechanics and effect authority |
| Presentation | behavioural decisions and effect execution |
| Tracing | source-of-truth domain state or ownership evidence |

A lower crate using an upper-layer term is an architecture smell even when no
Cargo dependency is present.

## Application-edge composition lock

`Application<C>` remains opaque over caller-chosen composition. The Stage 7
`ManagedConfigComposition<H>` is a proving consumer, not a canonical component
set and not precedent for accumulating every future capability in
`audiacore-application`.

Do not introduce service locators, dependency containers, global registries,
provider registries, manager frameworks, or implicit ambient context as the
system grows.

## External pattern references

The architecture is intentionally aligned with established patterns rather than
being treated as novel:

- AWS Prescriptive Guidance on hexagonal architecture describes ports as
  technology-agnostic abstractions and adapters as infrastructure-specific
  implementations, with the domain/application logic isolated from those
  implementation details: https://docs.aws.amazon.com/prescriptive-guidance/latest/hexagonal-architectures/overview.html
- Microsoft's Dependency Inversion and Explicit Dependencies principles require
  implementation details to depend on abstractions and collaborators to be
  passed explicitly instead of hidden behind global/infrastructure state:
  https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/architectural-principles
- `cap-std` documents `Dir` as a capability-based filesystem interface whose
  operations are relative to an acquired directory handle rather than a global
  filesystem namespace: https://docs.rs/cap-std/latest/cap_std/fs/
- Kubernetes controllers model the same desired-state/current-state separation
  used by the pure reconciliation layer: https://kubernetes.io/docs/concepts/architecture/controller/
- `tracing` documents the library/executable split used here: libraries emit
  instrumentation through `tracing`; subscriber installation belongs to the
  executable/application edge: https://docs.rs/tracing/latest/tracing/
- `cargo-deny` provides established dependency-graph checks for advisories,
  licenses, and trusted sources: https://embarkstudios.github.io/cargo-deny/checks/

External references confirm direction; they do not justify importing a stale or
unnecessarily broad dependency.

## Stage 8 acceptance

Validated implementation head: `240200cfa34a78b01ba6503814ec1d936412f791`  
Workflow run: `32624528904` (#436).

The full audit passed Ubuntu 24.04, macOS 15, Windows 2025, direct dependency
admission, the Stage 7 native end-to-end proof, and transitive supply-chain
policy. It found no layer inversion requiring production redesign. The only
post-cleanup failure was a stale Stage 7 expected-dependency assertion after an
unused dev edge was removed; the assertion was corrected to the accepted graph
rather than restoring the dependency.

The accepted architecture therefore remains deliberately small: effect-free
core/foundation semantics, explicit host ports and scopes, native effect
adapters, narrow capabilities, and an application composition/presentation/
observability edge. Future capabilities must earn their own boundaries and may
not weaken this lock merely for convenience.
