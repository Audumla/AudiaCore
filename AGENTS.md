# AudiaCore build rules

Canonical records:

- `docs/architecture/layer-lock.md` — current ownership/dependency rules;
- `docs/architecture/target-state.md` — required and deferred capabilities;
- `docs/architecture/roadmap.md` — proof sequence;
- `docs/architecture/revalidation.md` — historical acceptance evidence;
- `docs/architecture/dependencies.md` — dependency admission.

## Rules

- Build upward from proven contracts. New abstractions require a concrete consumer.
- Dependencies and vocabulary point downward. Lower layers do not acquire application/provider/plugin semantics.
- Sources provide data; resolution produces validated settings; application policy expresses intent; capabilities receive narrow requests.
- Policy never grants effect authority. Explicit authorities cross host contracts; native adapters perform effects.
- `audiacore-core` remains capability-neutral and effect-free. `Application<C>` is only an opaque composition seam, not a container framework.
- Reconciliation owns desired-versus-observed planning only.
- Managed Content is the canonical capability family. Its current implementation is only the whole-file slice; higher ownership semantics must be earned explicitly.
- Stable error-code prefixes own error category. Component `errors.yaml` files contain only canonical message and resolution.
- Keep domain events, operation evidence, tracing, execution output, status, and diagnostics distinct.
- Application/bootstrap composition, extension discovery, package resolution, and provider selection remain edge responsibilities. Do not introduce a service locator or runtime registry.
- First-party and external implementations must use the same typed capability/component seam after resolution.
- Do not add generic managers, global registries, ambient context, universal component lifecycle traits, or plugin frameworks without multiple concrete consumers.
- Direct third-party dependencies are admitted once at the workspace root and must be maintained, proportionate, platform-compatible, and correctly layered.
- Git history is the archive. Remove obsolete proving code, compatibility aliases, old terminology, generated artifacts, and duplicate instructions from the active tree.

Every production change must pass formatting, Clippy, tests, dependency admission, semantic layer gates, supply-chain checks, and Ubuntu/macOS/Windows CI.
