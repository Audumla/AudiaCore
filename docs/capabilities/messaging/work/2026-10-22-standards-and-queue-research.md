# Messaging standards and unified delivery — research ledger

**Status:** INVESTIGATING — architecture direction accepted, detailed contract not accepted.

**Question:** Which standard vocabulary and delivery contract let AudiaCore use one model for in-process and distributed messaging, without baking MQTT/AMQP/NATS into the engine?

## Verified current AudiaCore evidence

- [Events implementation](../../../../crates/audiacore-events/src/lib.rs) proves `EventEnvelope<E>`, `EventId`, `EventStreamId`, correlation/causation, monotonic ordering, bounded in-memory retention, and cursor paging.
- [Layer lock](../../../architecture/layer-lock.md) explicitly excludes a broker/queue from the current implementation.
- [Core boundary](../../../architecture/core-capabilities.md) establishes a **single** future messaging model with external protocol implementations.
- No queue acknowledgement, leased delivery, consumer subscriptions, broker transport, or durable store are currently proved in this repository.

## Prior exploration — source consolidation outstanding

Earlier AUDiaGentic gateway and collaboration discussions considered MQTT, NATS/JetStream, RabbitMQ, and a database-backed fallback, and emphasized persisted execution state, independently verified provider results, correlation, redelivery and recovery. In the old gateway design, a generalized event bus was **not** accepted as the execution authority.

That history is valuable **context, not a completed AudiaCore standard selection**. Before promoting any legacy conclusion, locate the original design/evidence in its original repo or history and record a stable reference here. Do not present conversational summaries as an implemented or formally benchmarked result.

## Standards candidates (preliminary; not a conformance assessment)

| Standard | Potential role | Unresolved question |
| --- | --- | --- |
| [CloudEvents](https://github.com/cloudevents/spec) | Event representation/interchange | How should requests, commands and work be represented **without** calling everything an event? |
| [AsyncAPI](https://www.asyncapi.com/docs/reference/specification/latest) | Contract/schema description | Can it describe our channels and message payloads without prescribing runtime architecture? |
| [AMQP 1.0](https://www.amqp.org/resources/specifications) | Delivery/settlement concepts and reference transport | Which settlement semantics are genuinely common to in-process and MQTT? |
| [MQTT 5](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html) | External transport implementation | QoS, ordering, shared subscriptions and redelivery do not automatically provide general durable work queues |
| NATS / JetStream | External transport implementation | What durability and consumer state remain core-neutral? |

No verified side-by-side standards comparison, implementation adapter tests, or acceptance matrix has yet been recorded in AudiaCore. **No standard has been selected as the canonical AudiaCore API.**

## Directional decisions — 2026-10-22

- **Accepted direction:** one AudiaCore messaging concept; do not create separate event and queue engines.
- **Accepted direction:** the application may select **zero, one, or multiple** implementations via explicit typed composition.
- **Accepted direction:** an optional basic in-process implementation may be maintained in the core repo; MQTT and other protocol implementations live in independent repos.
- **Not decided:** exact message envelope, channel addressing, ordering guarantees, API trait boundaries, ack/nack/lease model, persistence and backpressure semantics.
- **Not decided:** adopt CloudEvents directly, support projection at boundaries, or define a smaller transport-neutral envelope.
- **Not decided:** whether a common delivery trait can cover both pub/sub and work consumption without ambiguous guarantees.

## Next investigation (no new queue code yet)

1. Capture two real use cases: local component separation and durable gateway-to-provider work/result delivery.
2. Write acceptance tests for consumer acknowledgement, duplicate delivery, ordering, crash recovery, backpressure and event fan-out where each is actually required.
3. Compare in-process and MQTT behavior against those tests; do not promise identical guarantees unsupported by the transport.
4. Evaluate CloudEvents/AsyncAPI/AMQP mapping against required semantics.
5. Agree the smallest core contract and revise `audiacore-events` **instead of building a competing engine**.
6. Record the accepted design in the current [capability guide](../README.md) and global [core capability boundary](../../../architecture/core-capabilities.md); archive this research when no longer active.
