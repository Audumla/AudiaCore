# AudiaCore build rules

AudiaCore is a clean-room architecture revalidation and rebuild repository.

This file is repository guidance for external contributors and coding tools. It is not part of the AudiaCore product architecture and the word "agents" here does not imply an AI/agent runtime concept in Core.

Canonical architecture records:

- `docs/architecture/revalidation.md` — accepted proof history;
- `docs/architecture/layer-lock.md` — semantic ownership and dependency lock;
- `docs/architecture/target-state.md` — target capability map;
- `docs/architecture/roadmap.md` — post-Stage-8 proof sequencing;
- `docs/architecture/dependencies.md` — dependency decisions.

## Working rules

- Build upward one accepted layer at a time. Do not introduce later-layer abstractions early.
- Do not confuse an unimplemented target capability with a rejected capability. Required/deferred targets remain recorded in `target-state.md` until proved, superseded, split, or deliberately rejected.
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
- The current `audiacore-managed-config` crate is an accepted whole-file capability/proof. Managed Content is the target higher capability. Do not grow Managed Content into `audiacore-managed-config` by accretion, and do not rename the crate merely to imply semantics it does not yet implement.
- Reusable public failures expose stable coded identity. Component-owned `errors.yaml` catalogues own canonical human-facing message templates, kinds, and resolutions; typed local diagnostic context remains in Rust errors.
- Domain events, operation receipts, operational tracing, execution output/artifacts, public status projections, and explicit diagnostics are separate concepts.
- Libraries may emit structured tracing only when a later stage proves a need; executable/application edges own subscribers/exporters.
- Normal audit-level tracing uses stable structured fields and must not broadly format secrets, configuration objects, requests, contexts, or arbitrary error objects.
- `Application<C>` remains an opaque caller-chosen composition seam. Do not make `audiacore-application` a crate that imports and owns every future capability.
- Component selection, extension discovery, package resolution, and lifecycle belong at the application/bootstrap edge. Normal runtime consumers receive typed collaborators; they do not query a service locator.
- Capability, component, extension, package/source, and configured instance are distinct identities. Do not collapse them into one provider/plugin identifier.
- First-party and externally supplied implementations must use the same capability/component seam after resolution.
- External extension sources are a target capability. Do not assume a repository-local `plugins/` directory or runtime dynamic libraries are the required implementation mechanism.
- Before adding a direct dependency, verify that it is actively maintained or has credible current stewardship, is not archived/dead/superseded, has an acceptable security posture, and is proportionate to the required capability. Useful-but-stale dependencies are rejected.
- Recheck build-versus-buy before growing custom infrastructure that overlaps a maintained ecosystem library.
- Do not add global registries, service locators, generic manager frameworks, global event buses, runtime provider registries, or universal plugin abstractions.
- Git history is the archive. Do not retain obsolete implementations, compatibility debris, generated artifacts, local machine state, or duplicate instruction files in the tree.

## Acceptance

Every production stage must update the architecture record and target capability status, pass its unit/behaviour tests, pass dependency and semantic layer gates, and be green on Ubuntu, macOS, and Windows before the next stage is accepted.
