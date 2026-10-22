# Workflow

**Implemented:** `audiacore-workflow`. [Source and tests](../../../crates/audiacore-workflow/src/lib.rs)

Use `WorkflowDefinition` to decide deterministic transitions from current state and an event. `WorkflowInstance` tracks its own lifecycle/status and revision. `WorkflowTransition` describes `Continue`, `Complete`, or `Fail` plus effects **as data**. Snapshot and receipt types expose revision, status and effects.

**Not implemented here:** scheduling, persistence, transport, running effects, retries, durable worker queues, global workflow orchestrator. Those are higher application/runtime concerns; do not conflate a workflow event with a transport queue.
