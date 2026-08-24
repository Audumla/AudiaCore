# AudiaCore capability roadmap

Status: **POST-STAGE-8 PLANNING BASELINE**.

This document sequences future proof work against `target-state.md`. It does not
override the accepted Stage 8 `layer-lock.md`.

## Planning rule

For every proposed stage/slice:

1. identify the target capability it advances;
2. identify accepted lower contracts it consumes;
3. prove no lower layer acquires higher vocabulary/effects;
4. define the minimum concrete consumer that earns the abstraction;
5. reject registry/manager/framework machinery not required by that consumer;
6. update `target-state.md` when a target is proved, split, superseded, or rejected.

Stage 8 remains fixed unless a required target produces concrete evidence that a
Stage 8 primitive/boundary is insufficient.

## Stage 8 closeout

Stage 8 is complete when target-state recovery and target terminology are both
validated:

- `target-state.md` distinguishes proven, partial, required/deferred,
  hypothetical, and rejected capabilities;
- `layer-lock.md` records component/extension ownership without introducing a
  plugin framework;
- the acceptance record remains history rather than the future roadmap;
- `audiacore-managed-content` is the canonical capability family name;
- Stage 8 explicitly limits the implemented Managed Content scope to whole-file
  desired state;
- no `ManagedConfig*`, `audiacore-managed-config`, `managed_config`, or
  `IO-MCONFIG-*` compatibility surface remains in the active codebase;
- README/contributor guidance points to the correct architecture records;
- full cross-platform and supply-chain gates pass.

## Next proof — componentized application composition

Before plugin discovery/provider frameworks, prove the application edge can
compose at least two independent components without turning
`audiacore-application` into a crate that imports every capability.

Acceptance intent:

- retain `Application<C>` unchanged unless a concrete blocker is found;
- use explicit typed construction/injection;
- compose two independently useful capability implementations;
- no global registry, service locator, provider registry, or generic component
  manager;
- no universal `Component` lifecycle trait unless multiple consumers require the
  same lifecycle contract;
- reusable capability crates remain usable without extension machinery;
- application/bootstrap owns implementation selection and lifecycle;
- move/retire the Stage 7 `ManagedContentComposition<H>` proof shape if it would
  otherwise become the canonical all-capabilities composition.

The purpose is to earn the **component seam**, not the plugin system.

## Managed Content expansion

Managed Content already exists as a capability family; its Stage 8 implementation
is only the whole-file slice. Future slices extend `audiacore-managed-content`
without changing its layer ownership.

The first expansion should use one real AUDiaGentic requirement and establish the
smallest useful ownership model, such as one structured member or one bounded
text contribution, while preserving unrelated content.

Minimum direction:

- explicit contribution/resource identity;
- observe -> inspectable plan -> apply -> verify;
- parser-safe preservation of unrelated content;
- fail closed when ownership/input structure is ambiguous;
- operation receipt/evidence sufficient for later prune/restore reasoning;
- explicit authority separate from ownership semantics.

Do not attempt every format, patch type, multi-resource transaction, prune,
restore, and rollback mode in the first expansion. Whole-file mode remains a
valid Managed Content slice.

## Probe / observation

Introduce a narrow observation capability when a real application use case needs
observations that should not be hidden inside software lifecycle or Managed
Content. Candidate observations include file/directory presence, executable
version/discovery, package state, command outcome, content markers, and endpoint
reachability. Do not introduce a probe registry/provider framework.

## Software lifecycle

Build install/upgrade/uninstall/applicability/version semantics above
`ProcessHost` when one real integration requires them. Package-manager mechanics
do not belong in `ProcessHost`; shared dependency ownership remains distinct
from one application's integration state.

## Durable application execution

Reuse workflow/events/time primitives, but do not assume they already form the
finished execution system. A real proof may add durable state, resumability,
bounded public status, explicit diagnostics, output/artifact references,
retry/timer semantics where required, and compensation distinct from generic
workflow transition mechanics. Persistence abstractions are earned by a real
storage consumer.

## Extension composition

Only after the component seam is proven should extension identity/resolution be
introduced. First proof should prefer build/startup-time Rust composition from
an external crate/repository and demonstrate:

- external implementation satisfying the same contract as a built-in one;
- explicit extension identity/compatibility metadata;
- package/source identity separate from configured instance identity;
- validation before application composition;
- no service-locator access after composition.

Runtime dynamic loading is not required by this proof.

## Extension packaging/lifecycle

After a real external extension exists, prove only the deployment semantics
required: configured sources, package/version compatibility,
install/update/remove, integrity/provenance/security policy, and local/private
sources where needed. Choose dynamic libraries, out-of-process plugins, WASM, or
another transport only after concrete requirements justify one.

## Provider ecosystem

After component/extension composition is proven:

1. define provider capability vocabulary from a real provider;
2. add one concrete provider implementation;
3. integrate ACP transport/session semantics where applicable;
4. earn capability negotiation from real differences between at least two
   providers;
5. prove an external provider through the general extension seam.

Providers do not own generic orchestration, durable workflow state, plugin
discovery, or public status semantics.

## Interoperability surfaces

ACP, A2A, ASA, MCP, API, and control channels are projections/adapters around one
canonical application/execution model. Each must justify its edge vocabulary and
must not create a parallel task/workflow/authority/status model.

## Stage numbering

Do not assign permanent Stage 9+ numbers merely to fill a sequence. Number a
stage when its concrete consumer, boundary, exclusions, and acceptance proof are
ready.

The preferred next production proof is **componentized application composition**
because it protects the modular target before additional capabilities/providers
create pressure to accumulate dependencies in `audiacore-application`.
