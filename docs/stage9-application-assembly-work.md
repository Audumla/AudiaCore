# Stage 9 application assembly work record

Status: **PROVING**. Temporary record; remove after Stage 9 lock.

## Requirement

An application must be able to assemble typed Rust implementations without their repository or directory location becoming runtime semantics.

```text
local path / external Git source
        -> Cargo resolution
        -> typed implementation
        -> explicit bootstrap construction
        -> Application<C>
        -> normal typed runtime
```

## Hypotheses

1. Cargo path and exact-revision Git dependencies are sufficient for the first source-resolution requirement.
2. Existing AudiaCore contracts work unchanged across repository boundaries.
3. `Application<C>` is sufficient for explicit application composition.
4. Source resolution ends before normal runtime; no runtime registry/locator is required.
5. No reusable composition crate, universal `Component` trait, custom manifest, or package resolver is yet justified.

## Gate A — external-source proof

Use the existing `FileHost` contract.

- local implementation: standalone crate outside the AudiaCore workspace;
- external implementation: independent crate in another Git repository, pinned by exact commit;
- standalone proof app compiles/runs against each implementation without changing the AudiaCore contract;
- locked dependency resolution is required.

Acceptance: both sources satisfy the same typed contract; AudiaCore production code remains unchanged.

Reassessment: if Cargo and the existing contract are sufficient, do not add source-resolution infrastructure.

## Gate B — concrete application composition

Build a real application/bootstrap package with at least two independently useful collaborators, including one implementation obtained outside its own repository. Construct them explicitly into `Application<C>`.

Acceptance:
- explicit typed construction/injection;
- no service locator, runtime registry, string-keyed lookup, generic manager, or universal component lifecycle trait;
- package/source selection is absent from normal runtime code;
- application owns its concrete composition shape;
- AudiaCore gains code only if Gate A/B demonstrates a reusable missing primitive.

## Gate C — promotion/cleanup

After validation, classify every new construct as **KEEP**, **MOVE**, **SIMPLIFY**, **REMOVE**, or **DEFER**. Fold durable decisions into `target-state.md`, `layer-lock.md`, and `roadmap.md`; remove this work record and proof-only code.

Validation at each gate: format, Clippy/tests where applicable, locked Cargo builds, dependency/source inspection, and platform CI appropriate to the changed repository.