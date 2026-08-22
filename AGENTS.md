# AudiaCore build rules

AudiaCore is a clean-room architecture revalidation repository.

This file is repository guidance for external contributors and coding tools. It is not part of the AudiaCore product architecture and the word "agents" here does not imply an AI/agent runtime concept in Core.

The canonical semantic layer contract is `docs/architecture/layer-lock.md`.

## Working rules

- Build upward one accepted layer at a time. Do not introduce later-layer abstractions early.
- A new abstraction must have a concrete consumer or a stage-specific proof. Speculative framework code is rejected.
- Dependencies point downward only.
- Semantic ownership also points downward: a lower layer must not acquire vocabulary or behaviour owned by a higher layer merely because no Cargo dependency is present.
- Sources provide data; configuration resolution produces validated settings; application policy expresses behaviour intent; capabilities receive narrow requests. These are distinct concepts.
- Core remains capability-neutral and effect-free.
- Pure semantic libraries do not read environment variables, discover files, spawn processes, access networks, or own telemetry subscribers.
- Native effects cross narrow host contracts and live in concrete host implementations.
- Configuration is resolved at composition boundaries; capabilities receive validated typed requests derived from application policy, not configuration objects.
- Policy decides behaviour. Authority independently determines which effects are permitted. Configuration and policy cannot grant themselves effect authority.
- Reconciliation owns desired-versus-observed planning only; resource identity, ownership, paths, authority, and host semantics belong to consuming capabilities.
- The current managed-config crate is a whole-file capability. It does not prove ownership of pre-existing content and must not grow into the future higher-level Managed Content capability by accretion.
- Reusable public failures expose stable coded identity. Component-owned `errors.yaml` catalogues own canonical human-facing message templates, kinds, and resolutions; typed local diagnostic context remains in Rust errors.
- Domain events, operation receipts, operational tracing, and execution output are separate concepts.
- Libraries may emit structured tracing only when a later stage proves a need; executable/application edges own subscribers/exporters.
- Normal audit-level tracing uses stable structured fields and must not broadly format secrets, configuration objects, requests, contexts, or arbitrary error objects.
- Before adding a direct dependency, verify that it is actively maintained or has credible current stewardship, is not archived/dead/superseded, has an acceptable security posture, and is proportionate to the required capability. Useful-but-stale dependencies are rejected.
- Recheck build-versus-buy before growing custom infrastructure that overlaps a maintained ecosystem library.
- Do not add global registries, service locators, generic manager frameworks, global event buses, or universal provider/plugin abstractions.
- Git history is the archive. Do not retain obsolete implementations, compatibility debris, generated artifacts, local machine state, or duplicate instruction files in the tree.

## Acceptance

Every stage must update the architecture record, pass its unit/behaviour tests, pass dependency and semantic layer gates, and be green on Ubuntu, macOS, and Windows before the next stage is accepted.
