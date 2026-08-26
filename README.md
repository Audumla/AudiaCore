# AudiaCore

Clean-room Rust foundation for the AUDiaGentic platform.

Stages 0–9 are closed. The active workspace contains only earned reusable contracts and capabilities; proving scaffolding is retained in Git history rather than as production API.

Current layers and proven boundaries include:

- core identity and opaque caller-owned `Application<C>` composition seam;
- stable coded errors, sensitive values, templates, and configured error presentation;
- pure reconciliation and in-memory configuration resolution;
- explicit file/process host contracts and native adapters;
- events, workflow, and time primitives;
- Managed Content, currently limited to its whole-file desired-state slice;
- application-owned typed assembly, with standard Cargo path/exact-revision Git source resolution kept outside normal runtime semantics.

Architecture records:

- `docs/architecture/layer-lock.md` — current ownership/dependency rules;
- `docs/architecture/target-state.md` — target capability map;
- `docs/architecture/roadmap.md` — next proof order;
- `docs/architecture/revalidation.md` — accepted historical evidence;
- `docs/architecture/dependencies.md` — dependency policy and current decisions;
- `docs/architecture/stage9-application-assembly.md` — Stage 9 progression, source-equivalence proof, concrete consumer validation, and lock decisions.
