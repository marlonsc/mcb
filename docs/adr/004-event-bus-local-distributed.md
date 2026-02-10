---
adr: 4
title: Event Bus (Local and Distributed)
status: IMPLEMENTED
created: 
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Complete
---

## ADR 004: Event Bus (Local and Distributed)

## Status

**Implemented** (v0.1.1)

> Fully implemented with TokioEventBus (default) and NatsEventBus for distributed scenarios.

## Context

As the system became more modular, the need for asynchronous communication between components grew. For example, when a context provider updates data, other modules may need to react (logging, metrics, cache update, etc.) without direct coupling. For this, we evaluated implementing an internal EventBus. The requirements included: low latency for communication within the process and optionally distributing events between distinct server instances (in the future or in high availability scenarios). We considered in-house solutions (Tokio channels) and external messaging (such as NATS or RabbitMQ) for dissemination.

## Decision

We implemented an EventBus with two operational modes: by default, it uses asynchronous Tokio channels (Rust runtime) for publishing/subscribing to events within the same process; optionally, it supports integration with NATS (using JetStream) for publishing events in a distributed manner between multiple instances of Memory Context Browser. Concretely, we defined a common EventBus interface that has two implementations: TokioEventBus (default) and NatsEventBus (activated via configuration). The TokioEventBus uses structures such as broadcast or mpsc from Tokio for delivering events to local subscribers efficiently. The NatsEventBus, when enabled, connects to a NATS server (configurable via URL and credentials) and uses a specific topic/channel to send events, allowing different instances to synchronize their Actions (for example, multiple instances receiving context change notifications). The choice between Tokio or NATS is made at initialization time, reading configuration file settings (see Configuration section in README).

## Consequences

The EventBus introduction provides robust decoupling between modules: producers and consumers of events do not need to know each other, just publish or listen to certain event types. Internally, with TokioEventBus, performance impact is minimal thanks to Tokio's efficiency. When enabling NATS, scalability is gained, allowing Memory Context Browser to run in cluster and keep events synchronized, although introducing external dependency and network latency. This duality brings flexibility: in simple environments use the embedded mode, in distributed environments use NATS. In terms of maintenance, it adds the need to manage NATS configuration and ensure message schema compatibility between versions. Overall, the EventBus improves system responsiveness and extensibility for new functionalities (e.g., auditing, alerts, SSE streaming to admin panel clients). All decisions here documented were implemented in v0.1.1. Action required: evaluate in future ADR the use of other messaging backends (e.g., RabbitMQ) if requirements change, and formalize event protocols (message format, types) if the ecosystem expands.
