# Host access and native effects

**Implemented:** `audiacore-host` (contracts) and `audiacore-host-native` (standard OS implementation). [Contracts](../../../crates/audiacore-host/src/lib.rs) · [native adapter](../../../crates/audiacore-host-native/src/lib.rs)

`FileHost` exposes optional reads, whole-file writes and removal. `FileReadAuthority` and `FileWriteAuthority` independently define an explicit authorised root.

`ProcessHost` / `ProcessChild` describe process launch/ownership. `ProcessAuthority` is an executable-path allow-list; `ProcessRequest` carries program, arguments, environment and stdio policy.

The standard native adapter uses `cap-std` for capability-oriented filesystem access and native process operations.

**Important:** a file authority permits an effect, but does not prove ownership of a pre-existing file. Process launch authority is **not** a sandbox for the launched child.

Applications can supply another implementation of the narrow host traits without changing consuming capabilities.
