# Third-party dependency decisions

Status: Stage 8 dependency lock under validation.

Reviewed: 2026-08-23.

This record covers direct Rust crates, important transitive/security implications,
CI actions, and near-term libraries that could replace custom infrastructure.
The question is not only whether a dependency works. It must be maintained,
proportionate, compatible with AudiaCore's layer boundaries, and preferable to
both credible alternatives and writing/retaining custom infrastructure.

## Acceptance rule

A new third-party dependency is rejected unless all of the following are true:

- it has active maintenance or credible current organizational stewardship;
- it is not archived, abandoned, superseded, or on a clearly stale line;
- its security response and provenance are acceptable for its role;
- its license is compatible with the repository;
- its MSRV and Ubuntu/macOS/Windows support fit AudiaCore;
- its default and enabled feature surface is proportionate;
- its transitive dependency cost is understood;
- using it preserves the semantic layer boundary where it is introduced;
- credible alternatives, including using the standard library or no dependency,
  have been considered;
- the reason to use, defer, or reject it is recorded here.

Maintenance is judged in context. A mature stable crate does not need weekly
releases, but there must be credible evidence that security and compatibility
issues can still be handled. Popularity alone is not maintenance evidence.

## Locking model

AudiaCore uses two complementary locks:

1. `[workspace.dependencies]` is the single approval point for direct
   third-party Rust dependencies. Member crates inherit these declarations and
   may only add narrowly required features.
2. `Cargo.lock` is committed and CI uses `--locked`, fixing the exact resolved
   build including transitive crates.

Workspace version requirements are normal Cargo compatibility requirements, not
`=` pins. Exact pins in every manifest make legitimate ecosystem resolution
harder and do not replace the lockfile. A lockfile change is a dependency change
and requires review against this record.

GitHub Actions are dependencies too. Accepted actions must use immutable full
commit SHAs, with the human-readable release/version recorded in a comment.
Prefer removing an action when standard runner tooling provides the same narrow
function safely.

## Current direct Rust dependencies

### serde — KEEP

Role: typed serialization/deserialization boundary.

Why: Serde is the ecosystem-standard data model used by the TOML, JSON and YAML
adapters. Replacing it would increase coupling and custom conversion code.
Narrow alternatives such as miniserde do not provide an equivalent general
interface; format- or layout-specific systems such as borsh/rkyv solve different
problems.

Boundary: data conversion only. Serde must not become config source discovery,
application policy, or persistence semantics.

Accepted workspace line: `1.0.229`.

### serde_json — KEEP

Role: deterministic caller-supplied value/mapping tree for template context.

Why: the template capability genuinely needs nested maps, sequences and scalar
values plus stable JSON rendering of non-string values. Keeping `serde_json`
avoids inventing a generic value tree and numeric/string renderer. `simd-json`
is performance-specialized and materially heavier for a non-throughput-critical
presentation primitive.

Boundary: pure in-memory values only.

Accepted workspace line: `1.0.151`.

### toml — KEEP FOR PARSE/RESOLUTION

Role: parse already-acquired TOML into the current deterministic config resolver.

Why: it is the maintained canonical TOML implementation and the current manifest
disables default features, enabling only `std`, `serde`, and `parse`.

Alternative: `toml_edit` is the preferred established candidate when a future
Managed Content format adapter needs format-preserving TOML mutation. That is a
higher capability and is not a reason to introduce `toml_edit` into the config
resolver now.

Boundary: `audiacore-config` must not acquire files, environment, network sources,
or application policy.

Accepted workspace line: `1.1.4`.

### yaml_serde — KEEP, CONSTRAINED/WATCHED

Role: strict deserialization of caller-supplied owner-local `errors.yaml` files.

Why: it is the YAML Organization's maintained successor line to deprecated
`serde_yaml`, preserves mature Serde compatibility, and the current use is tiny,
local and schema-constrained.

Important transitive: `libyaml-rs`, maintained under the YAML Organization, is a
Rust translation of libyaml. This is a larger parser surface than the error
catalogue semantics themselves require, so YAML must not silently expand into a
general ingestion mechanism.

Alternatives considered:

- `serde_yaml`: REJECT — deprecated/unmaintained line.
- `yaml-rust2`: REJECT for new adoption — basic-maintenance line rather than the
  forward development path.
- `serde-saphyr`: WATCH — actively developed, pure Rust, forbids unsafe code and
  provides parsing budgets; currently newer and more maintainer-concentrated.
  Re-evaluate it if AudiaCore begins accepting untrusted/large YAML or if its
  stewardship matures further.
- changing `errors.yaml` to another format: DEFER — no architectural benefit is
  currently demonstrated that justifies format churn.

Boundary: error presentation metadata only; no file discovery or general config
source semantics.

Accepted workspace line: `0.10.7`.

### tracing — KEEP

Role: structured operational instrumentation at proven application edges.

Why: actively maintained in the Tokio ecosystem; spans and structured fields are
needed and are not supplied by the `log` facade. `slog` would introduce another
logging framework. OpenTelemetry is complementary export/semantic machinery, not
a replacement for in-process instrumentation.

Boundary: libraries do not own subscribers/exporters. Stable Audia fields are
kept small and OpenTelemetry conventions are reused where applicable.

Accepted workspace line: `0.1.44`.

### tracing-subscriber — KEEP AS DEV/EDGE SUPPORT

Role: Stage 7 proof subscriber and future executable/application-edge setup.

Why: canonical subscriber ecosystem for `tracing`. It must not move into pure or
host/capability layers.

Accepted workspace line: `0.3.23`.

## Evaluated but not current direct dependencies

### Figment — REJECT

The API fits layered configuration well, but its maintenance activity is too
stale for a new foundational dependency under AudiaCore's dependency-health
rule. API quality does not override maintenance risk.

### config-rs — DEFER / PRIMARY CONFIG-SOURCE CANDIDATE

Actively maintained and mature. It owns source collection/layering such as files
and environment. Current `audiacore-config` intentionally resolves
already-acquired content and records AudiaCore-specific exact ordered-input
provenance, so adopting config-rs inside that pure resolver would blur the source
boundary. Re-evaluate when a concrete application needs source acquisition.

### thiserror — DEFER

Actively maintained and suitable for deriving Rust error boilerplate, but it does
not replace stable `CodedError` identity or configured error presentation. Add it
only if hand-written conversion/display boilerplate becomes a demonstrated cost.

### miette — DEFER TO PRESENTATION EDGE

Useful for rich CLI diagnostics, not a replacement for AudiaCore error identity
or configured message catalogues. Introduce only with a real CLI/application
presentation consumer.

### cap-std — PROOF REQUIRED; STRONG ADOPTION CANDIDATE

Bytecode Alliance maintained, capability-oriented, cross-platform, no default
features, and directly addresses race-resistant path traversal/authority relative
filesystem access. Current `NativeFileHost` manually canonicalizes roots/parents,
checks symlinks and then performs `std::fs` operations, which leaves us owning a
security-sensitive problem already solved by a mature library.

Alternatives considered include `openat` (less portable/multi-component),
`pathrs` (Linux-specific), Unix-fd-oriented approaches, and retaining our custom
canonicalization. None currently has a better fit for the required
Ubuntu/macOS/Windows adapter.

Decision: do not add it as paperwork. Run a separate native-host proof showing
that `FileHost` contracts, authority separation, symlink escape prevention,
atomic replacement and all-platform behavior are preserved. If the proof passes,
prefer `cap-std` internally in `audiacore-host-native`; it must not leak into the
host contract or application layers.

Current candidate line reviewed: `4.0.3`.

## CI and supply-chain tooling

### actions/checkout — KEEP, UPGRADE AND SHA-PIN

The checkout action is justified because correct PR/ref checkout and GitHub
credentials are CI mechanics we should not reproduce. The previously used
floating `actions/checkout@v4` is not acceptable under this policy and its Node
runtime is deprecated on current runners.

Accepted target: checkout v7.0.1 at immutable commit
`3d3c42e5aac5ba805825da76410c181273ba90b1`, pending the CI-lock checkpoint.

### dtolnay/rust-toolchain — REMOVE

The project already has `rust-toolchain.toml` pinned to Rust 1.95.0, minimal
profile, rustfmt and clippy. GitHub hosted runners provide rustup. A direct
`rustup toolchain install` step is smaller and removes a third-party action
without losing semantics. Validate on all three runner platforms before the
removal is accepted.

### cargo-deny — CANDIDATE SUPPLY-CHAIN GATE

Actively maintained and combines RustSec advisories, license checks, source
restrictions and dependency bans. This is preferable to adding several
independent scanners if its configuration can remain narrow and deterministic.
If introduced, its action or binary version must itself be immutable/pinned.
Do not add it merely for a badge; first define the license/source/advisory policy
we expect it to enforce.

## Future review triggers

Re-open this record when any of the following occurs:

- a direct dependency or GitHub Action is added, removed or materially upgraded;
- a current dependency becomes archived, superseded or materially inactive;
- a security advisory affects an accepted line;
- enabled features or transitive dependencies materially expand;
- the MSRV/platform matrix changes;
- custom AudiaCore code grows into functionality already provided by a healthy
  established library;
- YAML/config/filesystem inputs become less trusted or substantially broader.

Stage 8 is not dependency-locked until the workspace centralization, CI action
pin/removal, current red semantic gate, and `cap-std` native-host decision have
all been independently validated on the supported platforms.
