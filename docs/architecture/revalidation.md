# AudiaCore clean-room architecture revalidation

Status: **ACTIVE**

AudiaCore rebuilds the production Rust foundation from an empty repository so every layer is re-earned rather than copied. Prior AUDiaGentic work is requirements evidence only; code, dependencies, API shape, and boundaries are reassessed at each stage.

## Method

Every stage must:

1. state the responsibility and deliberate exclusions before acceptance;
2. add only the minimum code needed to prove the responsibility;
3. add behaviour tests and architecture gates;
4. pass strict `cargo fmt`, Clippy `-D warnings`, and tests using the committed lockfile;
5. pass Ubuntu, macOS, and Windows;
6. record the accepted head and workflow run before the next stage is accepted.

A green build alone is insufficient. Dependency direction, effect ownership, stable errors, configuration provenance, policy/authority separation, and absence of speculative abstractions are acceptance criteria.

## Layer hypothesis under test

```text
future application/domain authority
              |
application capabilities
              |
native host implementation
              |
host contracts + authorities
              |
pure foundation semantics
              |
core
```

The clean-room rebuild may simplify this structure whenever a proposed boundary cannot justify itself.

## Global invariants

- Dependencies flow downward only.
- Core remains capability-neutral and effect-free.
- Pure foundation semantics remain deterministic and effect-free.
- Configuration acquisition belongs at an application edge; resolved configuration carries provenance.
- Policies are validated typed behaviour values and can be built from config or any other source.
- Authorities grant effects; config or policy does not implicitly grant authority.
- Native effects do not leak into semantic layers.
- One stable error code identifies one semantic condition.
- Domain events, operational tracing, and ordered execution output remain separate contracts.
- No service locator, global registry, generic manager layer, speculative provider framework, or abstraction without a proven consumer.

## Stage status

| Stage | Layer / proof | Status |
| --- | --- | --- |
| 0 | Repository + build discipline | ACCEPTED |
| 1 | Core | ACCEPTED |
| 2 | Stable error contract | ACCEPTED |
| 3A | Pure deterministic primitives | ACCEPTED |
| 3B | Configuration | ACCEPTED |
| 4A | File host contract + authority | ACCEPTED |
| 4B | Process host contract + authority | ACCEPTED |
| 5A | Native file effects + containment | ACCEPTED |
| 5B | Native process effects + lifecycle | ACCEPTED |
| 6A | Events capability | ACCEPTED |
| 6B | Workflow capability | IN PROGRESS |
| 6C | Time capability | BLOCKED BY 6B |
| 6D | Managed-config capability | BLOCKED BY 6C |
| 7 | Composition + policy + observability proof | BLOCKED BY 6D |
| 8 | Full layer-lock audit | BLOCKED BY 7 |

## Accepted checkpoints

### Stage 0 — repository/build discipline

Accepted head: `def74266e38e69553b3481978a74d9a13ed97f57`  
Workflow run: `32550330851` (#2) — Ubuntu/macOS/Windows passed.

Repository/toolchain/hygiene discipline existed before product code.

### Stage 1 — core

Accepted head: `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126`  
Workflow run: `32550518423` (#16) — Ubuntu/macOS/Windows passed.

Accepted only application/execution/correlation identity, identity-only `ExecutionContext`, and opaque `Application<C>`. Generic lifecycle, diagnostics, capability/component taxonomy, policy/authority, registries, configuration, I/O, tracing, serialization, runtimes, and provider semantics were rejected from core.

### Stage 2 — stable errors

Accepted head: `b87ed88b8c3d43efcb47d6564a5932505916479e`  
Workflow run: `32550714414` (#30) — Ubuntu/macOS/Windows passed.

Accepted zero-dependency `ErrorCode`, derived `ErrorCategory`, `ErrorDefinition`, `CodedError`, and repository-wide duplicate-code enforcement. No error registry, loader, logger, serializer, transport envelope, manager, or universal base-error hierarchy was introduced.

### Stage 3A — pure deterministic primitives

Accepted head: `1278f66bf720ed3bbb5d3c05cfb77f62ea5f8d55`  
Workflow run: `32551016872` (#52) — Ubuntu/macOS/Windows passed.

Accepted `audiacore-sensitive`, `audiacore-template`, and `audiacore-reconcile`. Each depends only on stable errors and owns no application identity or native effects.

### Stage 3B — configuration

Accepted head: `05031f8f5351381224d1848933ff577426ce98c7`  
Workflow run: `32552433778` (#72) — Ubuntu/macOS/Windows passed.

Accepted ordered in-memory TOML layers, recursive later-layer override, typed Serde resolution, exact ordered-input provenance revision, retained ordered layer identities, and coded failures.

Clean-room decisions:

- Figment was rejected; direct `serde + toml` is sufficient.
- Source discovery, filesystem/environment acquisition, remote configuration, and policy semantics remain outside the crate.
- `ResolvedConfig<T>` has no convenience API that silently discards provenance.
- Revalidation caught and fixed a `Default` provenance-initialization mismatch.

### Stage 4A — file host contract

Accepted head: `a467d1cff5ff8b5330e399c12409da08bafbab9f`  
Workflow run: `32552758268` (#90) — Ubuntu/macOS/Windows passed.

Accepted absolute `FileReadAuthority` / `FileWriteAuthority` and only `FileHost::read_optional`, `write`, and `remove`.

Clean-room reductions: no mandatory `read`, no lexical `allows(path)`, no list/watch/directory API, and no storage/service/manager abstraction. Canonicalization and safe containment belong to native implementation.

### Stage 4B — process host contract

Accepted head: `df7c6685b8e2048035ef70367ed7e0f9f7043ad6`  
Workflow run: `32553116555` (#106) — Ubuntu/macOS/Windows passed.

Accepted absolute executable allow-list authority, absolute requested executable/current directory, sensitive environment values, ambient environment disabled by default, explicit stdio modes, owned stream transfer, and direct-child `try_wait` / `wait` / `kill`.

Clean-room reductions: borrowed stdio accessors, one-shot `run`, generic async host futures, process managers/registries/schedulers, network host, and secret-provider host were rejected. Process authority is launch authority only, not a sandbox or descendant process-tree grant.

### Stage 5A — native file effects

Accepted head: `43453a48f5da0564f83aa56f381879f9bb710c7c`  
Workflow run: `32553515690` (#130) — Ubuntu/macOS/Windows passed.

Accepted canonical authority-root enforcement, existing-directory requirement, authority-relative resolution, tested parent/symlink escape rejection, optional reads, atomic replacement, and remove. Atomic durability is a private `file_store` module inside `audiacore-host-native`; no public/workspace file-store layer exists.

The portable implementation does not claim hostile-concurrent-filesystem race-proof sandboxing; that stronger claim would require a deliberate handle-relative/platform-specific design.

### Stage 5B — native process effects

Accepted head: `4baf0b3a491feb8b550f1e3ad4a82f40f1d15a16`  
Workflow run: `32554534159` (#154) — Ubuntu/macOS/Windows passed.

Accepted an isolated `process.rs` implementation with its own `NativeProcessError`; canonical requested-program/allow-list comparison; canonical existing working-directory validation; direct native stdio mapping; deny-by-default ambient environment with explicit sensitive insertion; owned child stream transfer; `try_wait`, `wait`, and `kill`; and best-effort direct-child kill+wait on dropped live handles.

Cross-platform tests use the test executable itself rather than shell-specific commands to prove unauthorized launch rejection, explicit environment insertion with ambient `PATH` cleared, piped output, working-directory rejection, lifecycle observation, termination, and reaping.

No generic cross-effect native error hierarchy, async runtime, provider/session abstraction, process manager, or descendant-tree containment claim was introduced.

### Stage 6A — events

Accepted head: `4fdbd7c7a0fa10e6a16af60db788f9cb3c81b088`  
Workflow run: `32555075146` (#178) — Ubuntu/macOS/Windows passed.

Accepted typed event/stream/causation identity; core correlation identity on envelopes; caller-owned monotonic sequence assignment with checked exhaustion; explicit `EventPolicy`; bounded/unbounded in-memory retention; retained iteration; and typed cursor paging with caught-up/expired/ahead semantics.

Cursor paging earns its place because bounded retention otherwise makes incremental observation ambiguous: a consumer must be able to distinguish an available cursor from evidence already evicted and from a cursor ahead of the stream.

Clean-room reductions:

- every `EventStream` is constructed with an explicit `EventPolicy`; there is no `EventStream::bounded(...)` shortcut;
- the redundant `after(sequence)` convenience iterator is omitted; `iter()` is the retained-view API and cursor paging is the incremental-read API;
- one neutral capability-local `EventError` replaces the misleading policy-returning-`EventStreamError` naming.

No event bus, broker, publisher/subscriber registry, queue, transport, retry engine, persistence, durable replay, scheduler, tracing, config, host, or native-effect dependency was introduced.

## Stage 6 — application capabilities

Capabilities are revalidated independently. They may use lower semantic contracts but may not acquire configuration, perform unmediated native I/O, own global runtime infrastructure, or turn policy into authority.

### Stage 6B — workflow

Required semantics:

- validated workflow instance identity;
- workflow-local status (`Running`, `Completed`, `Failed`) rather than a generic lifecycle abstraction in core;
- domain-owned deterministic transition decisions through a definition trait;
- effects emitted as data only;
- monotonic revision assigned by the workflow instance;
- optimistic revision check before domain logic;
- terminal-state rejection before domain logic;
- revision exhaustion check before domain logic and before state mutation;
- receipt carrying the new revision, status, and effects;
- an owned snapshot carrying id, revision, status, and cloned state so an application may persist/recover it without this capability owning storage.

Why snapshot semantics earn their place: workflow state/revision/status form one consistency boundary. An owned snapshot is the neutral checkpoint value an application can persist, serialize, compare, or transfer later without exposing storage or serialization concerns to the workflow capability.

Planned dependency boundary: `audiacore-workflow` depends only on `audiacore-errors`. It does not depend on core, events, config, host, native host, time, tracing, or a runtime.

Deliberate exclusions:

- no generic lifecycle framework;
- no event publication or dependency on `audiacore-events`;
- no persistence/store/repository abstraction;
- no scheduler, retry/backoff, clock, timer, async runtime, task execution, compensation engine, or global workflow registry;
- no provider/session/application semantics.

### Stage 6C — time

After 6B acceptance, revalidate caller-supplied timestamp/deadline semantics first and require the timer collection to justify itself independently; no clock, sleep, scheduler, task, or runtime ownership is assumed.

### Stage 6D — managed configuration

After 6C acceptance, revalidate the first capability that composes a semantic plan with host authority: optional observation, desired/observed reconciliation, ownership guard, and create/replace/delete application through `FileHost`. Parsing, watching, retries, scheduling, and multi-writer/CAS behaviour remain out unless separately proven.

## Stage 7 target

Only after all Stage 6 capabilities are individually accepted will the application-edge proof combine resolved configuration, typed policy, `Application<C>`, execution/correlation identity, native effects through authority, and real structured tracing. Policy must remain source-independent and tracing must remain an edge concern rather than becoming a semantic service.
