---
adr: 5
title: Context Cache Support (Moka and Redis)
status: IMPLEMENTED
created: 
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Complete
---

# ADR 005: Context Cache Support (Moka and Redis)

## Status

**Implemented** (v0.1.1)

> Fully implemented with MokaCache (default) and RedisCache for distributed scenarios.

## Context

To optimize performance of frequent read/write context operations, we decided to introduce a caching mechanism. Without cache, each request could Result in redundant calculations or accesses. We considered an in-memory solution for speed, and also the possibility of distributed cache for scenarios with multiple instances or greater persistence. We mapped two options: Moka (in-memory cache library, thread-safe, in Rust) for local use, and Redis (in-memory database/cache external) for shared cache.

## Decision

We implemented a configurable caching system, with Moka as the default provider and optional Redis support. The architecture defines a cache abstraction (for example, a trait CacheStore with operations get, set, invalidate, etc.), having MokaCache and RedisCache as implementations. By default, the application instantiates MokaCache, which offers high-performance local in-memory cache without external dependencies. If the configuration indicates Redis use (providing URL and connection parameters), the ServiceManager instead initializes a RedisCache and the modules pass through the same interface. The integration with the DI container (Shaku) allows injecting the chosen cache where needed, without the system components needing to know which implementation is in use.

## Consequences

The cache addition significantly improved performance in repetitive operations, reducing MCP Context Browser response latency, especially when consulted frequently by agents. With Moka, we obtained low latency and configuration simplicity (just adjusting size/TTL limits in configuration). When opting for Redis in distributed environments, we achieved cache consistency between instances and optional data persistence of cached context, at the cost of depending on more services (there may be increased operational complexity and external failure points). This flexibility attends various use cases: developers can start simple with Moka and scale to Redis as needed, maintaining unchanged application code. ADR pending: Formalize cache expiration and invalidation policies in a future architectural record. Currently, default policies (TTL, max size) are defined in code/config, but we recommend documenting them in detail and revisiting the cache strategy as data volume grows or new requirements emerge (e.g., LRU vs LFU, write-through behavior).
