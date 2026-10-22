# AudiaCore roadmap

Stages 0–9 are closed as foundation/proof milestones. Each later slice must prove the smallest real requirement, run the relevant validation matrix, then reassess every new abstraction before continuing. A planned abstraction is not automatically retained.

## 9. Application assembly — ACCEPTED PROOF

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
- a temporary external Rust proof consumer composed Managed Content/native host state and an event stream through an application-owned composition struct;
- that proof consumer passed rustfmt, Clippy, tests, and execution on all three platforms;
- `Application<C>` remains the core composition seam and gained only `composition_mut()` for explicitly mutating caller-owned composition state;
- no custom source resolver, plugin framework, runtime registry, service locator, dependency container, generic composition crate, or universal `Component` trait was justified.

The external consumer lived on the legacy AUDiaGentic line only to prove the boundary. Its integration PR was closed unmerged once the greenfield ownership boundary was clarified. It is not a production destination for the Rust rebuild.

Package install/update/remove, component compatibility metadata, declarative assembly above Cargo, dynamic loading, and WASM remain separate deferred concerns that require their own concrete consumer.

Proof progression, commits, workflow runs, and lock classification are preserved in `stage9-application-assembly.md`.

## Core capability boundary — CURRENT

AudiaCore is the reusable application platform, not a minimal kernel. Configuration, error handling/message catalogues, diagnostics, host access, reconciliation, Managed Content, messaging, workflow, time, and other broadly reusable application facilities remain core-repository concerns when expressed without protocol/vendor coupling.

ACP, MCP, A2A, MQTT transport, provider implementations, and similar integrations are separate repositories that depend on AudiaCore and are composed externally through Cargo.

AudiaCore must standardize one canonical messaging/event model. A future queue capability must evolve the current event semantics rather than introduce a second incompatible engine. See `core-capabilities.md`.

## Next implementation checkpoint — greenfield application/bootstrap

Build the first production application/bootstrap surface **inside this AudiaCore repository**, above the accepted foundation.

Acceptance:
- application-owned typed composition;
- explicit construction/injection into `Application<C>`;
- source selection remains Cargo/build metadata;
- no reuse of legacy AUDiaGentic application code as an implementation base;
- no service locator, dependency container, runtime registry, generic manager, or universal `Component` trait;
- re-evaluate whether any reusable application crate is actually earned only after the concrete implementation exists.

## 10. Managed Content higher slices

Use real content-management requirements to add only earned structured/partial ownership, preservation, prune/restore, verification, compensation, and evidence semantics.

## 11. Probe / observation

Introduce a sibling observation capability only for a real reusable observation need. No probe registry/provider framework.

## 12. Software lifecycle and managed package lifecycle

Add install/upgrade/uninstall/version semantics only when a real integration or application needs the platform to manage software/component packages. Keep package-manager mechanics out of host contracts and separate from application composition.

## 13. Durable execution

Compose workflow/events/time into application-level durability only as required by a real use case. Persistence, retries, compensation, status, diagnostics, and artifacts are earned independently.

## 14. Provider / ACP ecosystem — EXTERNAL MODULES

Build ACP and provider integrations in separate repositories against AudiaCore. If multiple external modules demonstrate a genuinely protocol-neutral provider/session contract that applications need to share, admit only that smallest contract into AudiaCore. Provider implementations, ACP negotiation, transport, and source discovery remain outside the core repository.

## 15. Interoperability surfaces — EXTERNAL MODULES

Build MCP, A2A, ASA, protocol control channels, and other integration surfaces as separately maintained repositories against the canonical AudiaCore application/execution/messaging model. They must not create parallel task, authority, workflow, messaging, status, or persistence models.
