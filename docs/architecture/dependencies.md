# AudiaCore dependency decisions

Status: Stage 8 dependency and supply-chain policy **ACCEPTED**.

Reviewed baseline: 2026-08-23. Current target capability implications are tracked
in `target-state.md`; this document records dependency admission and health
choices only.

## Admission rule

A new direct third-party dependency is rejected unless its role is concrete and
all of the following are acceptable:

- active maintenance or credible current organizational stewardship;
- no unresolved abandonment/deprecation/supersession concern;
- security response/provenance appropriate to the role;
- compatible license;
- supported MSRV and Ubuntu/macOS/Windows matrix;
- proportionate enabled feature and transitive surface;
- acceptable required transitive dependency health;
- correct semantic layer placement;
- credible alternatives, including std/no dependency, considered.

A useful API or popular crate is insufficient when maintenance/security or layer
fit is poor.

## Locking and governance

AudiaCore uses complementary controls:

1. root `[workspace.dependencies]` is the single approval point for direct
   third-party Rust dependencies;
2. member crates inherit approved dependencies with `workspace = true`;
3. direct local path dependencies must resolve to declared workspace members;
4. `Cargo.lock` is committed and CI uses the exact resolved graph;
5. `scripts/check-dependency-admission.py` covers normal/dev/build/target-specific
   direct dependency tables;
6. committed `deny.toml` plus SHA-pinned `cargo-deny` CI gates advisories,
   licenses, and dependency sources;
7. GitHub Actions are dependencies and accepted actions use immutable full SHAs.

Git dependencies remain rejected by the supply-chain policy unless deliberately
approved. A future external extension/package proof may justify changing that
policy, but extension sourcing must not silently bypass dependency admission.

## Current approved direct Rust dependencies

The current workspace approval set is:

| Dependency | Accepted role | Boundary |
| --- | --- | --- |
| `cap-std 4.0.3` | capability-relative filesystem implementation | `audiacore-host-native` only |
| `serde 1.0.229` | typed serialization/deserialization data boundary | no source/policy semantics |
| `serde_json 1.0.151` | deterministic JSON-like template context values | pure in-memory values |
| `toml 1.1.4` | parse already-acquired TOML for config resolution | no file/env/source acquisition |
| `tracing 0.1.44` | structured instrumentation at proven edges | no subscriber ownership in lower libraries |
| `tracing-subscriber 0.3.23` | executable/proof subscriber setup | edge/dev support only |
| `yaml_serde 0.10.7` | strict owner-supplied `errors.yaml` deserialization | error presentation metadata only |

No current approved direct dependency is intentionally retained after being
identified as dead or superseded.

## Current key decisions

### `cap-std` — KEEP, native host only

`cap-std` provides the capability-relative filesystem model required by the
native file adapter and is maintained in the Bytecode Alliance ecosystem. It
must not leak into host contracts, capability semantics, Managed Content, or
application policy.

### Serde / serde_json — KEEP

Serde remains the ecosystem-standard typed data boundary. `serde_json` remains a
small fit for explicit JSON-like template values and avoids inventing a generic
nested value tree.

Neither dependency grants config-source, persistence, or application-policy
semantics.

### `toml` — KEEP for current config resolution

The current resolver parses already-acquired TOML and records AudiaCore-specific
ordered provenance. It does not acquire files or environment sources.

`toml_edit` remains a plausible candidate for a future **Managed Content** TOML
adapter when format-preserving mutation becomes a real requirement. That target
is not a reason to add `toml_edit` to `audiacore-config` now.

### `yaml_serde` — KEEP narrowly, continue monitoring

The current YAML use is intentionally small and schema-constrained: caller-owned
error catalogue metadata only.

The Stage 8 review considered stronger/younger alternatives. `serde-saphyr`
remains the leading future challenger because of its defensive parser controls,
but its then-published required graph included stale `arraydeque 0.5.1`, which
failed AudiaCore's admission rule. Re-evaluate when that path is removed or
replaced.

`noyalib` and `saneyaml` remain watch/defer candidates rather than accepted
foundation dependencies because their 2026 projects/APIs were still too young
for this boundary. Original `serde_yaml`/`serde_yml` lines remain rejected as
unmaintained/deprecated choices.

A future Managed Content YAML editing requirement is a different dependency
selection problem from the present read-only error catalogue use.

### `tracing` / `tracing-subscriber` — KEEP at the edge

Libraries emit structured instrumentation only where a proven consumer requires
it. Subscriber/exporter installation belongs to an executable/application edge.
Do not introduce a logging/telemetry manager abstraction.

## Evaluated but not current direct dependencies

### Figment — REJECT

Its provider/layer/extraction API overlaps configuration needs, but the reviewed
maintenance activity was too stale for a new foundational dependency. API fit
does not override the maintenance gate.

### `config-rs` — DEFER / preferred config-source candidate

`config-rs` is actively maintained and mature. It is a better candidate than
building custom file/environment/other source-provider infrastructure when a
real application source-acquisition consumer exists.

Do not add it inside `audiacore-config` merely because config sources are a
target capability; source acquisition belongs at the application/source edge.

### `thiserror` — DEFER

Useful for Rust error boilerplate but unnecessary while hand-written typed errors
remain small. It would not replace stable `CodedError` identity or configured
error presentation.

### `miette` — DEFER to a real presentation consumer

Potentially useful for rich CLI presentation, but not part of error identity or
capability semantics.

## Extension/package dependency implications

The target state includes implementations sourced from other repositories or
locations. That does **not** mean foundation member crates should directly
path-depend on arbitrary folders or acquire ad hoc Git dependencies.

The initial extension proof should keep these concerns separate:

- capability/component contract;
- extension identity and compatibility metadata;
- package/source identity;
- build/startup composition;
- dependency/source admission.

If Cargo Git dependencies are selected for an external extension proof, update
both the direct-admission policy and `cargo-deny` source policy deliberately and
record the repository/revision trust model. Do not loosen source policy globally
without a concrete extension consumer.

Runtime dynamic libraries, WASM components, and out-of-process extensions are
not current dependency decisions.

## Build-versus-buy trigger

Reopen dependency evaluation before AudiaCore grows custom infrastructure that
overlaps a healthy maintained ecosystem project, especially for:

- config source acquisition;
- package/source resolution;
- structured document editing;
- protocol clients/transports;
- persistence;
- plugin/runtime loading;
- tracing/export integration.

The same rule applies in reverse: do not add a broad framework before a concrete
consumer demonstrates that a narrow in-house contract plus existing ecosystem
implementation is insufficient.

## Future review triggers

Re-open this record when:

- a direct dependency/action is added, removed, or materially upgraded;
- an accepted dependency becomes stale, archived, superseded, or vulnerable;
- enabled features/transitives materially expand;
- the MSRV/platform matrix changes;
- a target capability reaches implementation and needs an ecosystem library;
- an external extension proof requires a new Cargo/source trust model;
- `serde-saphyr` removes/replaces the stale transitive that blocked adoption;
- YAML/config/filesystem inputs become broader or less trusted.

The Stage 8 accepted dependency baseline is part of the foundation lock, but it
is not a prohibition on future dependencies. New dependencies must be earned by
real target capability work and admitted at the correct layer.
