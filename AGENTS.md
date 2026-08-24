# AudiaCore build rules

AudiaCore is a clean-room architecture revalidation and rebuild repository.

This file is guidance for contributors and coding tools, not a runtime/product
layer. The word "agents" here does not imply an AI/agent concept in Core.

Canonical architecture records:

- `docs/architecture/revalidation.md` — accepted proof history;
- `docs/architecture/layer-lock.md` — semantic ownership/dependency lock;
- `docs/architecture/target-state.md` — target capability map;
- `docs/architecture/roadmap.md` — post-Stage-8 proof sequencing;
- `docs/architecture/dependencies.md` — dependency decisions.

## Working rules

- Build upward one accepted layer at a time; do not introduce later abstractions early.
- Do not confuse an unimplemented target with a rejected capability. Keep required/deferred targets in `target-state.md` until proved, superseded, split, or deliberately rejected.
- New abstractions need a concrete consumer/stage-specific proof; speculative framework code is rejected.
- Dependencies and semantic ownership point downward only.
- Sources provide data; config resolution produces validated settings; application policy expresses behaviour intent; capabilities receive narrow requests.
- Core remains capability-neutral/effect-free.
- Pure semantic libraries do not acquire environment/files/process/network effects or telemetry subscribers.
- Native effects cross narrow host contracts and live in concrete adapters.
- Configuration is resolved at composition boundaries; capabilities receive typed requests, not config objects.
- Policy decides behaviour; authority independently permits effects. Config/policy cannot grant themselves authority.
- Reconciliation owns desired-versus-observed planning only; resource identity, ownership, paths, authority, and hosts belong to consuming capabilities.
- `audiacore-managed-content` is the canonical Managed Content capability family. Stage 8 implements only its whole-file slice. Target naming must not be interpreted as proof of partial/structured ownership semantics that are still deferred.
- Extend Managed Content through explicit later slices; do not recreate a separate `managed-config` capability or retain compatibility aliases for the old terminology.
- Reusable failures expose stable coded identity. Component-owned `errors.yaml` catalogues own human-facing definitions; typed local diagnostic context stays in Rust errors.
- Domain events, receipts/evidence, operational tracing, execution output/artifacts, public status, and explicit diagnostics are separate concepts.
- Executable/application edges own tracing subscribers/exporters.
- `Application<C>` remains an opaque caller-chosen composition seam. Do not turn `audiacore-application` into a crate that imports every future capability.
- Component selection, extension discovery, package resolution, and lifecycle belong at the application/bootstrap edge. Runtime consumers receive typed collaborators; they do not query a service locator.
- Capability, component, extension, package/source, and configured instance are distinct identities.
- First-party and external implementations use the same capability/component seam after resolution.
- External extension sources are a target. Do not assume a local `plugins/` directory or dynamic libraries are the required mechanism.
- Before adding a direct dependency, verify maintenance/stewardship, security/provenance, license, platform/MSRV, transitive cost, and proportionality.
- Recheck build-versus-buy before growing custom infrastructure that overlaps a maintained ecosystem library.
- Do not add global registries, service locators, generic manager frameworks, global event buses, runtime provider registries, or universal plugin abstractions.
- Git history is the archive. Do not retain obsolete implementations, compatibility debris, generated artifacts, local machine state, or duplicate instruction files.

## Acceptance

Every production stage must update architecture/target records, pass unit and
behaviour tests, pass dependency/semantic gates, and be green on Ubuntu, macOS,
and Windows before the next stage is accepted.
