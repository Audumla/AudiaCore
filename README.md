# AudiaCore

Clean-room Rust foundation used to revalidate the AUDiaGentic platform architecture from first principles.

The repository is intentionally built in explicit stages. A stage is accepted only when its design constraints, tests, architecture gates, and cross-platform CI are green before the next layer is introduced.

Stages 0 through 8 are accepted. See `docs/architecture/revalidation.md` for the acceptance record and evidence, and `docs/architecture/layer-lock.md` for the enduring semantic layer contract.
