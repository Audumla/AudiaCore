# AudiaCore clean-room architecture revalidation

Status: **ACTIVE**

AudiaCore rebuilds the production Rust foundation from an empty repository so every layer is re-earned rather than copied. Prior AUDiaGentic work is requirements evidence only; code and boundaries are reassessed at each stage.

## Method

For every stage:

1. State responsibility and deliberate exclusions.
2. Add only the minimum code needed to prove it.
3. Add behaviour tests and architecture gates before acceptance.
4. Validate on Ubuntu, macOS, and Windows.
5. Record the accepted commit/run before building upward.
6. Reassess whether the next proposed boundary is still justified.

A green build alone is insufficient: dependency direction, effect ownership, error semantics, configuration provenance, and absence of speculative abstractions are acceptance criteria.

## Stages

| Stage | Layer / proof | Status |
| --- | --- | --- |
| 0 | Repository + build discipline | ACCEPTED |
| 1 | Core | ACCEPTED |
| 2 | Stable error contract | ACCEPTED |
| 3A | Pure deterministic primitives | ACCEPTED |
| 3B | Configuration | ACCEPTED |
| 4A | File host contract + authority | ACCEPTED |
| 4B | Process host contract + authority | ACCEPTED |
| 5A | Native file effects + containment | IN PROGRESS |
| 5B | Native process effects + lifecycle | BLOCKED BY 5A |
| 6 | Application capabilities | BLOCKED BY 5 |
| 7 | Composition + policy + observability proof | BLOCKED BY 6 |
| 8 | Full layer-lock audit | BLOCKED BY 7 |

## Dependency hypothesis under test

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

A stage may simplify this if a boundary cannot justify itself.

## Global design invariants

- Dependencies flow downward only.
- Core is small, capability-neutral, and effect-free.
- Pure foundation semantics are deterministic and effect-free.
- Configuration acquisition belongs at an edge; resolved typed configuration carries provenance.
- Policies are validated typed behaviour values and can be built from config or other sources.
- Authorities grant effects and are not derived implicitly from policy/configuration.
- Native effects do not leak into semantic layers.
- Stable error codes identify one semantic condition each.
- Domain evidence, operational tracing, and ordered execution output are different contracts.
- No service locator, global registry, generic manager layer, speculative provider framework, or abstraction without a consumer.

## Accepted checkpoints

### Stage 0 — repository + build discipline

Accepted head: `def74266e38e69553b3481978a74d9a13ed97f57`  
Workflow run: `32550330851` (run #2) — Ubuntu/macOS/Windows passed.

**Decision:** repository/toolchain/hygiene discipline exists before product code.

### Stage 1 — core

Accepted head: `d626ef9886e4ad9eb8ae23f46ea8ee4b80e26126`  
Workflow run: `32550518423` (run #16) — Ubuntu/macOS/Windows passed.

**Accepted:** validated application/execution/correlation identity, identity-only `ExecutionContext`, opaque `Application<C>`.

**Rejected from core:** capability/component taxonomy, lifecycle, diagnostics/error codes, policy/authority, service registries, I/O, environment, runtime, tracing, serialization and provider semantics.

### Stage 2 — stable error contract

Accepted head: `b87ed88b8c3d43efcb47d6564a5932505916479e`  
Workflow run: `32550714414` (run #30) — Ubuntu/macOS/Windows passed.

**Accepted:** zero-dependency `ErrorCode`, derived `ErrorCategory`, static `ErrorDefinition`, optional `CodedError` and repository-wide duplicate-code enforcement.

**Rejected:** registry, runtime loader, logger, serializer, transport envelope, manager, core dependency and universal base-error hierarchy.

### Stage 3A — deterministic semantic primitives

Accepted head: `1278f66bf720ed3bbb5d3c05cfb77f62ea5f8d55`  
Workflow run: `32551016872` (run #52) — Ubuntu/macOS/Windows passed.

**Accepted:**

- `audiacore-sensitive`: explicit sensitive wrapper, deterministic redaction and coded key validation;
- `audiacore-template`: tiny named-slot parsing/rendering with coded failures;
- `audiacore-reconcile`: desired/observed planning where effect intent is data.

Each crate depends only on `audiacore-errors`. None depends on core or owns filesystem/environment/process/network/runtime/telemetry behaviour.

### Stage 3B — configuration

Accepted head: `05031f8f5351381224d1848933ff577426ce98c7`  
Workflow run: `32552433778` (run #72) — Ubuntu/macOS/Windows passed.

**Accepted:** explicit ordered in-memory TOML layers, recursive later-layer override, typed Serde resolution, exact-input provenance revision, retained ordered layer identities and stable coded parse/resolution failures.

**Dependency reassessment:** the clean-room rebuild rejects Figment. `audiacore-config` uses only `audiacore-errors`, `serde`, and `toml` with the narrow parsing/Serde surface required by the contract. Source discovery, filesystem/environment acquisition and policy semantics remain outside the crate.

**Additional correction found by revalidation:** derived `Default` would have initialized provenance differently from `ConfigLayers::new()`. `Default` now delegates to `new()` and a regression test locks that invariant.

### Stage 4A — file host contract

Accepted head: `a467d1cff5ff8b5330e399c12409da08bafbab9f`  
Workflow run: `32552758268` (run #90) — Ubuntu/macOS/Windows passed.

**Accepted:** separate read/write authority values with absolute roots and a file effect contract containing only `read_optional`, `write`, and `remove`.

**Clean-room reductions from the previous implementation:** mandatory `read` is omitted because no accepted capability consumes it; there is no lexical `allows(path)` API; relative authority roots are rejected so grants do not silently depend on process CWD.

Authority values describe grants only. Canonicalization, root existence, symlink handling and safe containment remain responsibilities of the concrete effect implementation.

### Stage 4B — process host contract

Accepted head: `df7c6685b8e2048035ef70367ed7e0f9f7043ad6`  
Workflow run: `32553116555` (run #106) — Ubuntu/macOS/Windows passed.

**Accepted:** explicit absolute executable allow-list; absolute requested executable and optional working directory; deny-by-default ambient environment inheritance; sensitive environment values; explicit stdio modes; owned child lifecycle with ownership-only `take_stdin`, `take_stdout`, and `take_stderr`; `try_wait`, `wait`, and direct-child `kill`.

**Clean-room reductions from the previous implementation:** borrowed stdio accessors are omitted; no generic async future/runtime boundary exists; there is no one-shot process `run`, process manager, scheduler, registry, network host, or secret-provider host.

`ProcessAuthority` is launch authority only. It does not claim to sandbox the child or constrain descendant process, filesystem, network, or account authority after launch.

## Stage 5 — native host implementation

Stage 5 is split by effect family so native filesystem safety can be proven independently of process execution.

### Stage 5A — native file effects + containment

The concrete implementation may depend on `audiacore-host` only. It owns native filesystem effects and implementation errors; low-level native errors are **not** forced into the stable coded-error contract until a reusable capability projects them across its public boundary.

Required semantics under test:

- canonicalize the authority root at the effect boundary and require it to be an existing directory;
- relative target paths resolve under the authority root, never process CWD;
- absolute target paths are accepted only when their canonical/effective location remains within the canonical authority root;
- existing reads canonicalize the target and reject escapes;
- a missing optional-read returns `None` only when its existing parent canonicalizes inside the authority root;
- writes require an existing canonical parent inside the authority root;
- a symlink leaf is never replaced by a managed write/remove operation;
- replacement is performed through a uniquely created same-directory temporary file, file `sync_all`, atomic `rename`, and parent-directory `sync_all` on Unix;
- temporary-file collision is retried without deleting another owner's file;
- temporary files are cleaned on failed writes;
- remove is authority-mediated and uses the same write-target validation.

### Clean-room boundary decision

There will be **no public or workspace-level `file-store` crate**. Atomic durability is a private implementation detail inside `audiacore-host-native`; it has no independent semantic consumer and therefore does not earn a public layer boundary.

### Security claim boundary

The native implementation must reject ordinary `..`/absolute/symlink path escapes at the operation boundary. Using portable `std::fs` path operations does **not** prove a race-free filesystem sandbox against a hostile actor concurrently replacing path components. AudiaCore will not claim that stronger guarantee until a platform-specific handle-relative design is deliberately implemented and tested.

### Acceptance tests

At minimum, the three-OS stage must prove optional missing read, create, overwrite, read, remove, parent escape rejection, authority-root validation, and temporary-file collision handling. Unix additionally proves directory-symlink and leaf-symlink escape rejection. Overwrite is exercised on Windows as an explicit cross-platform replacement proof.

### Stage 5B — native process effects + lifecycle

Blocked until Stage 5A is accepted. It will independently prove executable canonicalization/authorization, deny-by-default environment inheritance, explicit sensitive environment insertion, stdio mapping, owned stream transfer, wait/try-wait/kill, and direct-child cleanup without claiming descendant process-tree containment.
