# Time and timers

**Implemented:** `audiacore-time`. [Source and tests](../../../crates/audiacore-time/src/lib.rs)

`Timestamp::from_millis()` and `Deadline::at()` use caller-supplied absolute values. `TimerSet` supports `arm()`, `cancel()`, `next_deadline()`, and `drain_due(now)`, which deterministically returns due timer IDs.

The caller owns clock acquisition and scheduling; this crate never reads the system clock, sleeps, spawns tasks or starts a scheduler.
