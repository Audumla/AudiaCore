# Stable errors

**Implemented:** `audiacore-errors`. [Source and tests](../../../crates/audiacore-errors/src/lib.rs)

Use `ErrorCode` and `CodedError` for stable machine-readable identity. Error category derives from the code prefix and is **not separately configured**.

Errors describe an actual failure independently from how that failure is presented or logged. Keep message/resolution text in the [presentation capability](../presentation/README.md), which can supply caller-owned catalogues.

**Not implemented here:** ambient logging, reporting pipeline, global error registry, transport-specific error mapping. The diagnostics/logging facility is a core target, but has not yet been implemented.
