# AudiaCore dependency decisions

Status: **ACCEPTED FOUNDATION POLICY**.

## Admission

A direct dependency is admitted only when a concrete consumer requires it and maintenance/stewardship, security/provenance, license, MSRV/platform support, transitive cost, and layer placement are acceptable.

Controls:
- root `[workspace.dependencies]` is the direct third-party approval point;
- members inherit approved dependencies with `workspace = true`;
- local path dependencies in this workspace must resolve to workspace members;
- `Cargo.lock` is committed and normal CI uses `--locked`;
- `scripts/check-dependency-admission.py` covers direct dependency tables;
- `deny.toml` plus SHA-pinned `cargo-deny` gates advisories, licenses, and sources;
- accepted GitHub Actions use immutable SHAs.

## Current direct dependencies

| Dependency | Role | Boundary |
| --- | --- | --- |
| `cap-std 4.0.3` | capability-relative filesystem implementation | `audiacore-host-native` only |
| `serde 1.0.229` | typed deserialization | data boundaries only |
| `serde_json 1.0.151` | explicit mapping values for templates | `audiacore-template` |
| `toml 1.1.4` | parse already-acquired config content | `audiacore-config` |
| `yaml_serde 0.10.7` | strict caller-supplied error catalogue parsing | `audiacore-error-catalog` |

No dependency is retained solely because an historical proof used it.

## Stage 9 source policy

The AudiaCore foundation workspace remains free of Git dependencies. Stage 9 proves external sourcing from a **standalone application outside this workspace**, first with a local path and then with a Git dependency pinned to an exact revision.

That proof does not relax foundation dependency admission and does not imply AudiaCore needs a custom source resolver. Reopen this policy only if a later AudiaCore production consumer itself requires a new dependency/source class.

## Deferred candidates

- `config-rs`: reconsider when a real application needs file/environment/other config source acquisition. Source acquisition stays outside `audiacore-config`.
- `toml_edit` or another maintained structured editor: reconsider when a concrete Managed Content slice requires format-preserving mutation.
- `tracing` / `tracing-subscriber`: re-admit at an executable/application edge when a real consumer needs structured operational tracing.
- dynamic loading/WASM/IPC libraries: select only after runtime extension deployment/isolation requirements are concrete.

Reopen this record whenever a direct dependency/action is added, removed, materially upgraded, or becomes stale/vulnerable, or when a target capability reaches implementation and needs ecosystem support.
