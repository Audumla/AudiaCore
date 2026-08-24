# AudiaCore

Clean-room Rust foundation used to revalidate and rebuild the AUDiaGentic platform architecture from first principles.

Stages 0 through 8 are accepted and remain the foundation baseline. Future work builds upward from that baseline; required target capabilities may be documented before their implementation is earned.

Architecture documents have distinct roles:

- `docs/architecture/revalidation.md` — accepted Stage 0–8 proof history and evidence;
- `docs/architecture/layer-lock.md` — enduring semantic ownership and dependency rules;
- `docs/architecture/target-state.md` — required, proved, deferred, hypothetical, and rejected target capabilities;
- `docs/architecture/roadmap.md` — sequencing rules for proving missing target capabilities;
- `docs/architecture/dependencies.md` — dependency admission and health decisions.

The current `audiacore-managed-config` crate is an accepted narrow whole-file desired-state capability. **Managed Content** is the higher target capability and has not yet replaced that crate in code.
