# Stage 8 smoke application

This is a deliberately small consumer application built on top of the accepted Stage 8 AudiaCore foundation. It lives on a separate branch and is a standalone Cargo workspace so it does not become a member of, or change the lockfile for, the accepted foundation workspace.

The vertical slice is:

```text
embedded TOML source
  -> audiacore-config resolution + provenance
  -> ManagedConfigRequest
  -> Application<ManagedConfigComposition<NativeFileHost>>
  -> managed-config observe / plan / apply
  -> FileHost + explicit read/write authority
  -> NativeFileHost / cap-std
  -> temporary hello.txt
```

The app executes the same request twice. The first execution must return `Created`; the second must return `Noop`. It then reads the target through `FileHost`, verifies the resolved bytes, prints an `AUDIACORE_SMOKE_OK` marker, and removes the temporary directory.

The only direct `std::fs` operations are executable/test-harness bootstrap and cleanup of the temporary authority root. The managed target itself is created, observed, and reconciled through AudiaCore host/capability boundaries.

Run from the repository root:

```sh
cargo run --manifest-path examples/stage8-smoke-app/Cargo.toml
```
