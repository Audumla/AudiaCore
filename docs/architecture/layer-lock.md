# AudiaCore layer lock

Status: **ACCEPTED FOUNDATION CONTRACT**.

## Governing rule

Sources provide data. Resolution produces validated settings. Application policy expresses application intent. Capabilities receive narrow requests. Reconciliation computes change. Authority permits effects. Host contracts describe effects. Native adapters perform effects. Results describe what happened. Presentation explains it. Tracing observes it.

No lower layer may acquire vocabulary, dependencies, source acquisition, authority, or orchestration owned above it.

## Current layers

| Layer | Owns | Must not own |
| --- | --- | --- |
| `audiacore-core` | application/execution/correlation identity; opaque caller-owned `Application<C>` with immutable/mutable composition access | capabilities, policy, authority, I/O, providers, extensions, runtime |
| `audiacore-errors` | stable error code and prefix-derived category | messages, resolutions, diagnostics, I/O |
| `audiacore-sensitive` | sensitive values, secret-key identity, explicit redaction helpers | logging, stores, discovery, policy |
| `audiacore-template` | pure mapping-only message rendering | object traversal, discovery, I/O |
| `audiacore-error-catalog` | caller-owned message/resolution definitions + source provenance | category duplication, global registration, discovery, I/O |
| `audiacore-reconcile` | desired/observed -> effect intent | resource identity, ownership, authority, hosts, policy |
| `audiacore-config` | ordered in-memory TOML resolution + provenance | file/env/network acquisition, policy, authority |
| `audiacore-host` | narrow file/process contracts + explicit scope descriptors | application policy, recipes, content ownership, native mechanics |
| `audiacore-host-native` | OS file/process mechanics | application/provider/config/content semantics |
| `audiacore-events` | currently proven event/message envelope, correlation/causation, ordering, retention and cursor semantics | permanent separate queue engine, broker-specific semantics, MQTT transport |
| `audiacore-workflow` | deterministic workflow transitions/checkpoints | runtime, persistence, scheduler, compensation engine |
| `audiacore-time` | caller-supplied timestamps/deadlines/timer sets | clocks, sleeps, scheduler, runtime |
| `audiacore-managed-content` | Managed Content capability; currently whole-file observe/plan/apply | config acquisition, native I/O, application policy, unproved partial ownership |

There is intentionally **no current application-assembly crate**. Stage 9's external proof consumer demonstrated that application-owned typed composition plus `Application<C>` is sufficient for the proved requirement. Reusable capabilities remain independent; bootstrap code chooses concrete implementations and owns the composition shape. The greenfield production application/bootstrap surface belongs in this repository above the foundation and must be implemented independently of the legacy AUDiaGentic application line. A reusable assembly layer must not be introduced until multiple concrete applications demonstrate common semantics beyond explicit Rust construction.

## Effect and authority boundary

Policy decides desired behaviour. Authority independently permits effects. Configuration and policy cannot mint authority.

`FileReadAuthority`, `FileWriteAuthority`, and `ProcessAuthority` are explicit scope descriptors. Native filesystem containment is enforced after acquiring a `cap_std::fs::Dir`; process authority is a launch allow-list, not a child sandbox.

## Managed Content

Managed Content is the canonical capability family. The current slice supports only whole-file optional bytes:

```text
observed optional bytes + desired optional bytes
        -> pure plan
        -> noop | create | replace | delete
        -> explicit file authority
```

Partial/structured ownership, contribution identity, prune/restore, multi-resource coordination, verification, compensation, and receipts remain higher slices. Target naming does not imply those semantics already exist.

## Error contract

Stable error code is the single source of category. Category is derived from the code prefix; configured presentation must not restate it.

Owner-local `errors.yaml` contains only:

```yaml
CODE:
  message: "..."
  resolution: "..."
```

New code categories are added in `audiacore-errors` only when the platform target requires a distinct stable category.

## Application assembly and external sources

Stage 9 proved this boundary with locked path and exact-revision Git dependencies on Ubuntu, macOS, and Windows:

```text
Cargo/package metadata
        -> source resolution
        -> concrete typed implementation
        -> explicit application/bootstrap wiring
        -> Application<C>
        -> source-agnostic runtime
```

External source/package resolution happens before bootstrap composition. Once resolved, built-in, local, and external implementations satisfy the same typed contracts; normal runtime code receives collaborators and does not know their source location.

Application/bootstrap edges own implementation selection, compatibility validation where required, concrete wiring, provider selection, and observability setup. Package install/update/remove and runtime-loading mechanisms are separate higher concerns and must not be folded into composition by default.

Do not introduce service locators, dependency containers, global registries, generic managers, universal component lifecycle traits, runtime provider registries, or plugin frameworks without concrete evidence. Stage 9 specifically found none of these necessary. Validation evidence is recorded in `stage9-application-assembly.md`.

## Canonical messaging direction

AudiaCore will have one canonical asynchronous messaging/event model. Domain events, commands, requests, responses, and queued work must not become separate infrastructure engines with incompatible envelopes, ordering, correlation, or consumption semantics.

The current `audiacore-events` crate is an accepted partial proof, not a permanent boundary against a future queue subsystem. Before queue/broker support is implemented, its proven semantics must be reassessed and folded into the single AudiaCore messaging contract.

Transport implementations such as MQTT, NATS, RabbitMQ, Redis, or other brokers live in separate repositories and implement the AudiaCore contract. Broker-specific wire/QoS/connection semantics do not belong in AudiaCore unless a transport-neutral application requirement proves a common semantic.

See `core-capabilities.md` for the repository boundary.

## External module boundary

AudiaCore is the reusable application platform. Configuration, errors, presentation, diagnostics, host access, reconciliation, Managed Content, messaging, workflow and time are core-platform concerns when expressed as protocol-neutral application capabilities.

ACP, MCP, A2A, MQTT transport, provider-specific adapters, and other protocol/vendor integrations are separate repositories. They depend on AudiaCore; AudiaCore does not depend on them.

## Cross-cutting outputs

Domain events, operation receipts/evidence, operational tracing, execution output/artifacts, public status, and explicit diagnostics remain separate concepts.

Tracing dependencies are not part of the current foundation because the historical application proof is no longer an active production layer. A real executable/application consumer may re-admit the maintained tracing ecosystem at that edge.
