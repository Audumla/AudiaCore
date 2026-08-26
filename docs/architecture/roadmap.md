# AudiaCore roadmap

Stages 0–9 are closed. Each later slice must prove the smallest real requirement, run the relevant validation matrix, then reassess every new abstraction before continuing. A planned abstraction is not automatically retained.

## 9. Application assembly — ACCEPTED

Stage 9 proved application-owned typed composition with external source acquisition kept outside normal runtime semantics.

Accepted boundary:

```text
Cargo/package metadata
        -> source resolution
        -> concrete typed implementations
        -> explicit application/bootstrap construction
        -> Application<C>
        -> normal source-agnostic runtime
```

Accepted outcomes:

- local path and exact-revision Git implementations can satisfy the same AudiaCore contract with identical application source;
- locked source-equivalence proof passed Ubuntu 24.04, macOS 15, and Windows 2025;
- a concrete AUDiaGentic Rust bootstrap composed Managed Content/native host state and an event stream through an application-owned composition struct;
- the concrete locked bootstrap passed rustfmt, Clippy, tests, and execution on all three platforms;
- `Application<C>` remains the core composition seam and gained only `composition_mut()` for explicitly mutating caller-owned composition state;
- no custom source resolver, plugin framework, runtime registry, service locator, dependency container, generic composition crate, or universal `Component` trait was justified.

Package install/update/remove, component compatibility metadata, declarative assembly above Cargo, dynamic loading, and WASM remain separate deferred concerns that require their own concrete consumer.

Proof progression, commits, workflow runs, and lock classification are preserved in `stage9-application-assembly.md`.

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
