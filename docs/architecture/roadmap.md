# AudiaCore roadmap

Stage 8 is closed. Each later slice must prove the smallest real requirement, run the full validation matrix, then reassess every new abstraction before continuing. A planned abstraction is not automatically retained.

## 9. Application assembly

Target: build an application from typed implementations regardless of whether they are built in, in another local directory, or in an external repository/package. Source resolution happens before bootstrap composition; normal runtime receives typed collaborators only.

Use standard Rust/Cargo mechanisms first. Do not build a custom package resolver, plugin runtime, registry, container, or universal `Component` trait unless later proofs show a concrete gap.

### 9.0 Contract lock

Docs only.

- make external sourcing part of the first composition proof;
- distinguish source resolution from package lifecycle and runtime loading;
- keep `Application<C>` opaque;
- keep Stage 8 production code unchanged.

### 9.1 External source proof

Build a standalone application outside the AudiaCore workspace using an existing AudiaCore contract and at least one implementation outside the application/AudiaCore repository boundary.

Prove both:
- local path dependency;
- Git dependency pinned to an exact revision.

Acceptance:
- no AudiaCore production change unless the proof demonstrates a real missing contract;
- reproducible locked builds;
- external and local implementations use the same typed contract;
- source acquisition remains an application/build concern.

Reassess whether Cargo already solves the required source-resolution problem before adding anything.

### 9.2 Typed composition proof

Compose at least two independently useful components, including one externally sourced implementation, through explicit construction into `Application<C>`.

Acceptance:
- typed construction/injection;
- no service locator, runtime registry, generic manager, or string-keyed dependency lookup;
- no reusable composition crate unless multiple real consumers demonstrate common semantics;
- lifecycle abstractions only if the components actually require shared lifecycle behaviour.

### 9.3 Source equivalence and identity

Prove that built-in/local-path/external-source implementations can satisfy the same contract without changing normal runtime code.

Keep package/source identity separate from configured-instance identity. Add explicit component/compatibility metadata only when Cargo/package metadata cannot express a demonstrated requirement.

### 9.4 Declarative assembly

Only if a real application needs it, allow an application definition to select sources/implementations and produce an explicit build/bootstrap graph.

Prefer build-time assembly. Runtime-loaded native libraries, WASM, or out-of-process extension transports remain separate hypotheses.

### 9.5 Stage 9 lock

Stop and audit the result before further capability work. Classify every new construct as **KEEP**, **MOVE**, **SIMPLIFY**, **REMOVE**, or **DEFER**. Re-run formatting, Clippy, tests, dependency admission, semantic gates, supply-chain checks, Ubuntu/macOS/Windows CI, and reproducible external-source builds.

## 10. Managed Content higher slices

Use real content-management requirements to add only earned structured/partial ownership, preservation, prune/restore, verification, compensation, and evidence semantics.

## 11. Probe / observation

Introduce a sibling observation capability only for a real reusable observation need. No probe registry/provider framework.

## 12. Software lifecycle and managed package lifecycle

Add install/upgrade/uninstall/version semantics only when a real integration or application needs the platform to manage software/component packages. Keep package-manager mechanics out of host contracts and separate from application composition.

## 13. Durable execution

Compose workflow/events/time into application-level durability only as required by a real use case. Persistence, retries, compensation, status, diagnostics, and artifacts are earned independently.

## 14. Provider / ACP ecosystem

From real ACP-capable providers, establish typed provider/session contracts and capability negotiation. Provider implementations do not own generic orchestration or source discovery.

## 15. Interoperability surfaces

Add MCP, A2A, ASA, API, and control channels as adapters/projections around the canonical application/execution model. Do not create parallel task, authority, workflow, status, or persistence models.
