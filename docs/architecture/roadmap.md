# AudiaCore capability roadmap

Status: **POST-STAGE-8 PLANNING BASELINE**.

This document sequences future proof work against `target-state.md`. It is not
an implementation backlog and does not override the accepted Stage 8
`layer-lock.md`.

## Planning rule

For every proposed stage or slice:

1. identify the target capability it advances;
2. identify which accepted lower contracts it consumes;
3. prove that no lower layer must acquire higher-layer vocabulary or effects;
4. define the minimum concrete consumer that earns the abstraction;
5. reject registry/manager/framework machinery that is not required by that
   consumer;
6. update `target-state.md` when a target is proved, superseded, split, or
   rejected.

The accepted Stage 8 foundation remains fixed unless a required target
capability produces concrete evidence that a Stage 8 primitive or boundary is
insufficient.

## Immediate documentation gate — complete before new production capability work

The target-state recovery is complete when:

- `target-state.md` distinguishes proved, required/deferred, partial,
  hypothetical, and rejected capabilities;
- `layer-lock.md` records composition/extension ownership without introducing a
  plugin framework;
- the Stage 8 acceptance record remains historical rather than acting as the
  future roadmap;
- Managed Content is recorded as the target higher capability while
  `audiacore-managed-config` remains accurately described as the current
  whole-file proof;
- README/contributor guidance points to the correct document for current state,
  target state, layer rules, roadmap, and dependency decisions.

## Next proof — componentized application composition

Before introducing plugin discovery or provider frameworks, prove that the
application edge can compose at least two independent components without turning
`audiacore-application` into a crate that imports every capability.

Acceptance intent:

- retain `Application<C>` unchanged unless a concrete blocker is found;
- use explicit typed construction/injection;
- compose two independently useful capability implementations;
- no global registry, service locator, provider registry, or generic component
  manager;
- no universal `Component` lifecycle trait unless both consumers demonstrably
  require the same lifecycle contract;
- reusable capability crates remain independently usable without extension
  machinery;
- application/bootstrap code owns implementation selection and lifecycle;
- move or retire the Stage 7 `ManagedConfigComposition<H>` proving shape if it
  would otherwise become the canonical all-capabilities composition type.

The purpose of this proof is to earn the **component seam**, not the plugin
system.

## Managed Content proof

Managed Content is the target successor capability for application-owned
contributions to external content. It should be proved as a higher capability,
not by expanding `audiacore-managed-config` until that crate changes meaning.

Initial proof should use one real AUDiaGentic requirement and establish the
smallest valuable ownership model, for example one structured member or one
bounded text contribution, while preserving unrelated content.

Minimum direction:

- explicit contribution/resource identity;
- observe -> inspectable plan -> apply -> verify;
- parser-safe preservation of unrelated content;
- fail closed when ownership or input structure is ambiguous;
- operation receipt/evidence sufficient for later prune/restore reasoning;
- explicit authority remains separate from ownership semantics.

Do not attempt every format, patch type, multi-resource transaction, or rollback
mode in the first slice.

The existing managed whole-file capability may remain a lower mechanism if that
is useful; its long-term retention is decided by the Managed Content proof, not
by naming preference.

## Probe / observation proof

Introduce a narrow observation capability only when a real application use case
needs observations that should not be hidden inside software lifecycle or
Managed Content.

Candidate observations include file/directory presence, executable discovery,
version state, package state, command outcome, content markers, and endpoint
reachability.

Do not introduce a probe registry or generic provider framework.

## Software lifecycle proof

Build software/package lifecycle above `ProcessHost` after one real integration
requires install/upgrade/uninstall/applicability/version semantics.

Keep package-manager mechanics out of `ProcessHost`. Shared dependency ownership
must remain distinguishable from one application's integration state.

## Durable application execution

Reuse the accepted workflow/events/time primitives, but do not assume that they
already constitute the finished AUDiaGentic execution system.

A durable execution proof should establish only the missing application-level
semantics required by a real use case, such as:

- durable execution identity/state;
- resumable or inspectable transition state where required;
- bounded status projection;
- explicit diagnostics separated from status;
- output/artifact references rather than large inline payload duplication;
- retry/timer semantics only where the use case proves them;
- compensation semantics distinct from generic workflow transition mechanics.

Persistence/repository abstractions must be earned by the first durable storage
consumer rather than introduced speculatively.

## Extension composition proof

Only after the component seam is proven should the project introduce extension
identity and resolution.

First proof should prefer build/startup-time Rust composition from an external
crate/repository. It should demonstrate:

- an external implementation satisfying the same contract as a built-in one;
- explicit extension identity and compatibility metadata;
- source/package identity kept separate from configured instance identity;
- validation before application composition;
- no runtime service-locator access after composition.

Do not require runtime dynamic loading for this proof.

## Extension packaging/lifecycle

After at least one external extension exists, prove the package/source lifecycle
actually required by deployment:

- configured source representation;
- package/version compatibility;
- install/update/remove where needed;
- integrity/provenance/security policy;
- local/private source support where required.

Choose runtime loading technology only after these requirements are concrete.
Rust dynamic libraries, out-of-process plugins, and WASM components are separate
options with different ABI, isolation, security, and distribution trade-offs.

## Provider ecosystem

Once component and extension composition are proven, establish the provider
capability contract from a real ACP-capable provider.

Sequence intent:

1. provider capability vocabulary and request/result boundary;
2. one concrete provider implementation;
3. ACP transport/session integration where applicable;
4. capability negotiation from real differences between at least two providers;
5. external provider extension proof using the general extension seam.

Provider implementations do not own generic application orchestration, durable
workflow state, plugin discovery, or status semantics.

## Interoperability surfaces

ACP, A2A, ASA, MCP, API, and control channels should be added as projections or
adapters around the canonical application/execution model.

Each surface must prove why its vocabulary belongs at the edge and must not
create a parallel task/workflow/authority/status model.

## Stage numbering

Do not assign permanent Stage 9+ numbers merely to fill a sequence. Number a
stage when its concrete consumer, boundary, exclusions, and acceptance proof are
ready.

The current preferred next production proof is **componentized application
composition** because it protects the intended modular target before additional
capabilities or providers create pressure to accumulate dependencies in
`audiacore-application`.
