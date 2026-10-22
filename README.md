# AudiaCore

Clean-room Rust application foundation for the AUDiaGentic platform.

Stages 0–9 are closed as foundation/proof milestones. The active workspace contains only earned reusable contracts and capabilities; proving scaffolding is retained in Git history rather than as production API.

Current layers and proven boundaries include:

- core identity and opaque caller-owned `Application<C>` composition seam;
- stable coded errors, sensitive values, templates, and configured error presentation;
- pure reconciliation and in-memory configuration resolution;
- explicit file/process host contracts and native adapters;
- one canonical messaging/event foundation, plus workflow and time primitives;
- Managed Content, currently limited to its whole-file desired-state slice;
- application-owned typed assembly, with standard Cargo path/exact-revision Git source resolution kept outside normal runtime semantics.

[Documentation index](docs/README.md) · [Current capability guides](docs/capabilities/README.md)

Architecture records:

- `docs/architecture/core-capabilities.md` — what belongs in AudiaCore versus separately maintained implementation/protocol repositories;
- `docs/architecture/layer-lock.md` — current ownership/dependency rules;
- `docs/architecture/target-state.md` — target capability map;
- `docs/architecture/roadmap.md` — next proof order;
- `docs/architecture/revalidation.md` — accepted historical evidence;
- `docs/architecture/dependencies.md` — dependency policy and current decisions;
- `docs/architecture/stage9-application-assembly.md` — Stage 9 progression, source-equivalence proof, external proof-consumer validation, and lock decisions.
