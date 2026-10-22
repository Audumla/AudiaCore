# Messaging and events

**Implemented now:** `audiacore-events`, a caller-owned ordered in-memory event stream. [Source and tests](../../../crates/audiacore-events/src/lib.rs)

Use `EventStream<E>` with `EventStreamId` and `EventPolicy`; append events using `EventId`, `CorrelationId`, and optional `CausationId`. Read via `iter()` or `page_after(EventCursor, limit)`. Sequence numbers are monotonic. Bounded retention can expire old cursors.

**This is not yet a queue/broker:** no subscribers, publication, acknowledgements, durable delivery, retries, or transport abstraction are implemented.

## Target direction — not yet an API

AudiaCore must have **one canonical message/delivery model**, rather than a second queue engine alongside a separate event engine. That model should carry events, commands, requests, responses and work with their distinct semantics. An application may compose zero, one, or multiple implementations, such as standard in-process delivery and independently maintained MQTT/NATS transports. The same core semantics apply across them, though transport guarantees can differ.

An in-process default may live in this repository as an **optional crate**. External implementations are obtained through Cargo and explicitly wired by application bootstrap. No automatic runtime implementation override is implied.

Detailed research, decisions, and open questions: [active messaging research ledger](work/2026-10-22-standards-and-queue-research.md). Global ownership decisions: [core capabilities](../../architecture/core-capabilities.md).
