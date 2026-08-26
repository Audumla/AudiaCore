# Stage 9 application assembly validation

Status: **ACCEPTED PROOF RECORD**.

This document preserves the Stage 9 progression and validation evidence. It is historical evidence, not an active implementation plan. Current ownership rules remain in `layer-lock.md`; current capability status remains in `target-state.md`.

## Requirement proved

An application can assemble typed Rust implementations without repository/package location becoming normal runtime semantics:

```text
local path / exact-revision Git source
        -> Cargo resolution
        -> concrete typed implementation
        -> explicit application/bootstrap construction
        -> Application<C>
        -> normal typed runtime
```

Package/source resolution therefore belongs at the build/bootstrap edge. It is not a core capability and does not justify a runtime registry, service locator, dependency container, generic manager, universal `Component` trait, or custom package resolver.

## Gate A — source equivalence

The proof used the existing `audiacore-host::FileHost` contract and identical application source in two standalone applications outside the AudiaCore workspace:

- local-path implementation;
- implementation from a separate AUDiaGentic Git commit pinned to an exact revision.

Both variants executed the same Managed Content flow:

```text
observe -> plan -> apply -> Created
observe -> plan -> apply -> Noop
```

The proof explicitly checked that the two `main.rs` files were identical and that both dependency graphs were locked.

Accepted proof commit: `f77726491ddfbf2acd451aed8bc19aaf389c3dd2`.

Accepted workflow: `stage9-source-proof` run #8 / `32922723155`.

Platform result:

| Platform | Identical source | Locked local path | Locked exact-revision Git |
| --- | --- | --- | --- |
| Ubuntu 24.04 | pass | pass | pass |
| macOS 15 | pass | pass | pass |
| Windows 2025 | pass | pass | pass |

The external fixture was subsequently removed from the live proof branch; its exact proof commit remains available through Git history.

## Gate B — concrete application composition

The real consumer was implemented in AUDiaGentic under `apps/audiagentic-bootstrap`.

Its application-owned composition contains two independently useful collaborators:

```text
Application<AudiagenticComposition>
        |
        +-- BootstrapState
        |     +-- NativeFileHost
        |     +-- explicit file authorities
        |     +-- ManagedContentTarget
        |     +-- observe/plan/apply/verify
        |
        +-- EventStream<BootstrapEvent>
```

Normal Rust source contains no Git URL, revision, path-source selection, registry lookup, or package resolver logic. Source selection exists in Cargo metadata only.

Concrete consumer commit: `ea383a2fd9c68316d5c72cda837c9ad20d6d3ba1` in `Audumla/AUDiaGentic`.

Concrete validation: `ci-rust-bootstrap` run #2 / `32923571192`.

On Ubuntu 24.04, macOS 15, and Windows 2025 the gate passed:

- exact AudiaCore revision verification;
- rustfmt;
- Clippy with warnings denied;
- locked tests;
- locked executable run.

The executable proved first reconciliation `Created`, second reconciliation `Noop`, and two appended bootstrap events.

## Core seam earned by the proof

Gate B exposed one reusable omission: `Application<C>` owned mutable collaborators such as `EventStream`, but exposed only immutable composition access. Requiring every consumer to introduce interior mutability or a container merely to mutate its own composition would add false runtime semantics.

Stage 9 therefore added only:

```rust
pub const fn composition_mut(&mut self) -> &mut C
```

with a core unit test.

Implementation commit: `c3b0ee87a965e522c00874611e6e947689c8584a`.

The source proof remained green after that change: run #9 / `32922928494`.

No source resolver, composition crate, component framework, provider registry, or runtime lookup mechanism was added to AudiaCore.

## Progression and corrections

The proof was intentionally staged before permanent implementation. It surfaced build/proof mechanics without changing architecture:

1. nested standalone Cargo packages initially inherited the parent workspace; each proof package was made an explicit standalone workspace instead of adding proof crates to AudiaCore's production workspace;
2. source resolution was first validated unlocked, then lockfiles were generated and the same matrix rerun with `--locked`;
3. the concrete application gate initially found only a rustfmt line-wrap difference; formatting was corrected and the unchanged design then passed all three platforms;
4. unrelated AUDiaGentic Python canonical-provider-ID failures were kept out of Stage 9 rather than being hidden or opportunistically changed.

These corrections did not require new AudiaCore capability abstractions.

## Stage 9 lock classification

| Construct | Decision | Reason |
| --- | --- | --- |
| `Application<C>` | KEEP | sufficient opaque composition seam |
| `Application::composition_mut()` | KEEP | earned by real mutable owned collaborators |
| application-owned typed composition struct | KEEP | explicit wiring without framework semantics |
| Cargo path dependencies | KEEP | sufficient local source mechanism |
| exact-revision Cargo Git dependencies | KEEP | sufficient external source proof mechanism |
| committed application `Cargo.lock` | KEEP | reproducible resolution |
| custom source/package resolver | REMOVE / NOT JUSTIFIED | Cargo solved the proved requirement |
| generic composition crate | REMOVE / NOT JUSTIFIED | only application-specific wiring exists |
| universal `Component` trait | REMOVE / NOT JUSTIFIED | no common lifecycle semantics were demonstrated |
| service locator / dependency container / global registry | REJECTED | unnecessary runtime indirection |
| package install/update/remove | DEFER | software/package lifecycle is a separate capability |
| dynamic library or WASM loading | DEFER | runtime loading remains a separate hypothesis |
| declarative assembly above Cargo | DEFER | no demonstrated requirement exceeds Cargo metadata yet |

## Accepted invariant

Stage 9 establishes this boundary:

```text
Cargo/package metadata owns source resolution
                -> application/bootstrap owns concrete selection and wiring
                -> Application<C> owns typed collaborators
                -> normal runtime is source-location agnostic
```

Future extension/provider work must preserve that boundary unless a concrete deployment requirement proves that build-time composition is insufficient.