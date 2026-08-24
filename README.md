# AudiaCore

Clean-room Rust foundation for the AUDiaGentic platform.

The accepted Stage 8 baseline is intentionally small. The active workspace contains only earned reusable contracts and capabilities; historical proving scaffolding is retained in Git history, not as production API.

Current layers:

- core identity and opaque `Application<C>` composition seam;
- stable coded errors, sensitive values, templates, and configured error presentation;
- pure reconciliation and in-memory configuration resolution;
- explicit file/process host contracts and native adapters;
- events, workflow, and time primitives;
- Managed Content, currently limited to its whole-file desired-state slice.

Architecture records:

- `docs/architecture/layer-lock.md` — current ownership/dependency rules;
- `docs/architecture/target-state.md` — target capability map;
- `docs/architecture/roadmap.md` — next proof order;
- `docs/architecture/revalidation.md` — accepted historical evidence;
- `docs/architecture/dependencies.md` — dependency policy and current decisions.
