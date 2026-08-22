# AudiaCore build rules

AudiaCore is a clean-room architecture revalidation repository.

## Working rules

- Build upward one accepted layer at a time. Do not introduce later-layer abstractions early.
- A new abstraction must have a concrete consumer or a stage-specific proof. Speculative framework code is rejected.
- Dependencies point downward only.
- Core remains capability-neutral and effect-free.
- Pure semantic libraries do not read environment variables, discover files, spawn processes, access networks, or own telemetry subscribers.
- Native effects cross narrow host contracts and live in concrete host implementations.
- Configuration is resolved at composition boundaries; capabilities receive validated typed policy values, not configuration objects.
- Policy decides behaviour. Authority determines which effects are permitted.
- Reusable public failures use stable codes with canonical messages and resolutions while retaining typed local error context.
- Domain events, operational tracing, and execution output are separate concepts.
- Libraries may emit structured tracing only when a later stage proves a need; executable/application edges own subscribers/exporters.
- Do not add global registries, service locators, generic manager frameworks, global event buses, or universal provider/plugin abstractions.
- Git history is the archive. Do not retain obsolete implementations, compatibility debris, generated artifacts, local machine state, or duplicate instruction files in the tree.

## Acceptance

Every stage must update `docs/architecture/revalidation.md`, pass its unit/behaviour tests, pass architecture gates, and be green on Ubuntu, macOS, and Windows before the next stage is accepted.
