# AudiaCore core capability boundary

Status: **TARGET ARCHITECTURE — REVISABLE BY CONCRETE PROOF**.

AudiaCore is the reusable Rust application platform on which applications and separately maintained protocol/integration modules are built. It is not intended to be a minimal kernel that omits normal application infrastructure.

The repository owns capabilities that are broadly required to build non-trivial applications and that must remain protocol-, vendor-, and deployment-neutral. Protocols, brokers, providers, and other external integrations depend on AudiaCore; AudiaCore must not depend on them.

## Core capability rule

A capability belongs in AudiaCore when it establishes reusable application semantics or baseline application infrastructure that multiple independently built applications/modules need to agree on.

Typical core responsibilities include:

- application, execution, correlation, and composition identity;
- configuration acquisition, resolution, validation, and provenance;
- stable errors, presentation, sensitive-value handling, logging, tracing, and diagnostics;
- authority and host contracts plus the standard native host implementation;
- reconciliation and generic desired-state capabilities such as Managed Content;
- one canonical asynchronous messaging/event model;
- workflow, time, timers, and durable-execution primitives as they are earned;
- generic observation/probe facilities when concrete use cases establish their contract;
- explicit application/bootstrap composition.

A capability does not leave AudiaCore merely because an alternative implementation can exist elsewhere. Core defines the stable contract and may provide the standard/default implementation. Independently maintained crates can provide alternative implementations of the same contract.

## Optional standard implementations

**Repository membership is not an application dependency.** AudiaCore may maintain optional standard implementation crates (for example native host adapters, file configuration sources, or in-process messaging) alongside its canonical contracts. An app opts into those crates through Cargo; it does not have to link or construct them.

For each capability, **zero, one, or multiple implementations** must be valid whenever the application semantics permit it. For example, an app may have no messaging, only in-process messaging, only MQTT messaging, or local and MQTT messaging simultaneously. The app explicitly wires concrete implementations; there is no implicit runtime override, universal container, or auto-selection of a default.

Standard local implementations and external transports implement **the same capability contract**, without guaranteeing identical broker-specific delivery properties. Source location does not determine runtime semantics.

See [current capability guides](../capabilities/README.md) for implemented APIs and usage; those guides intentionally separate current behavior from active research.

## External module rule

A capability belongs outside the AudiaCore repository when its primary purpose is integration with a particular protocol, broker, provider, vendor, service, or ecosystem.

Examples:

- ACP;
- MCP;
- A2A;
- MQTT transport;
- NATS, RabbitMQ, Redis, or other broker transports;
- provider-specific adapters;
- Discord, Zulip, and similar service integrations;
- other protocol/vendor-specific surfaces.

These repositories depend inward on AudiaCore contracts and are composed by applications through normal Cargo dependencies.

```text
external module / implementation
            |
            v
       AudiaCore contracts
            |
            v
   shared application semantics
```

AudiaCore never acquires a dependency on an external protocol module merely to support that module.

## One messaging model

AudiaCore must not grow separate event and queue engines.

Events, commands, requests, responses, and queued work are different message semantics carried through one canonical application messaging model. The common model should own only semantics that implementations must agree on, such as identity, correlation/causation, ordering where required, addressing/stream identity where required, delivery state where required, and consumption progress.

```text
domain event
command
request / response
work item
        |
        v
canonical AudiaCore messaging contract
        |
        +--> standard in-process implementation
        +--> external MQTT implementation
        +--> external NATS implementation
        +--> other implementations
```

The current `audiacore-events` crate is accepted proof of event identity, envelopes, correlation/causation, sequencing, retention, and cursor semantics. It is **not** a second permanent engine alongside a future queue subsystem. Before queue/broker capability is added, these proven pieces must be reassessed as inputs to the single canonical messaging contract.

MQTT-specific concepts, wire format, QoS mechanics, connection management, broker discovery, and client implementation remain outside AudiaCore. An MQTT crate may implement the AudiaCore messaging contract.

The messaging contract must be derived from concrete application requirements rather than by copying one broker's API.

## Current core repository capabilities

The following are currently accepted as belonging in AudiaCore:

| Capability | Current crate/state |
| --- | --- |
| application/execution identity and typed composition | `audiacore-core` |
| stable coded errors | `audiacore-errors` |
| sensitive values and redaction primitives | `audiacore-sensitive` |
| message/error templates and configured presentation | `audiacore-template`, `audiacore-error-catalog` |
| configuration resolution and provenance | `audiacore-config`; source acquisition still to be built |
| pure desired/observed reconciliation | `audiacore-reconcile` |
| explicit filesystem/process authority and contracts | `audiacore-host` |
| standard native filesystem/process implementation | `audiacore-host-native` |
| canonical messaging foundations | currently `audiacore-events`; contract revision still required before queue/broker work |
| deterministic workflow primitives | `audiacore-workflow` |
| deterministic time/timer primitives | `audiacore-time` |
| generic Managed Content capability | `audiacore-managed-content` |
| repository-level cross-capability validation | `tests/smoke` |

Logging/tracing/diagnostics, configuration source acquisition, broader messaging delivery semantics, observation, and durable execution remain expected AudiaCore capabilities whose exact contracts must be earned by real application use.

## Composition model

Separately maintained implementations are combined by the consuming application through Cargo and explicit typed construction.

```text
AudiaCore
  + external ACP crate
  + external MQTT crate
  + external provider crate
  + other implementation crates
              |
              v
       Cargo source resolution
              |
              v
      explicit typed bootstrap
              |
              v
        Application<C>
```

No runtime plugin framework, service locator, dependency container, global implementation registry, or universal component lifecycle trait is implied by repository separation.

## Boundary review rule

This document is a target boundary, not a permanent enumeration of every future crate.

Before adding or extracting a capability:

1. identify the concrete application requirement;
2. decide whether the semantics are broadly application-foundational or integration-specific;
3. if foundational, extend AudiaCore through the smallest stable contract;
4. if integration-specific, implement it in a separate repository against AudiaCore;
5. prove independent build/test and external Cargo composition;
6. revise this boundary if concrete evidence shows the classification is wrong.
