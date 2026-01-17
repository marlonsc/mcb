# ADR 002: Dependency Injection with Shaku

## Status

**Implemented** (v0.1.1)

> Fully implemented across all 7 crates with Shaku DI container and ServiceManager integration.

## Context

To integrate the various modules in crates while maintaining low coupling, we considered adopting a dependency injection (DI) container. DI allows service implementations to be resolved at runtime or through composition, facilitating implementation swapping (e.g., mocks in tests, or different providers for the same interface). We analyzed DI options in Rust and design patterns manual. The Shaku library stood out for providing compile-time DI, with support for singleton components and transient providers, meeting the project's needs.

## Decision

We implemented dependency injection via Shaku as the base for composing MCP Context Browser services. Each crate defines a Shaku module specifying its components (#[derive(Component)]) and interfaces (dyn Trait) that need to be resolved. The ServiceManager works in conjunction with Shaku to build the application's main module, registering concrete implementations of each interface provided by the crates. For example, if there are different context providers implementing the same trait, all can be registered and resolved according to configuration. We use Shaku components for singleton services (single instance, e.g., central managers) and Shaku providers for services where each resolution generates a new instance (if applicable).

## Consequences

The use of Shaku brought decoupling and architectural flexibility. We can add new services or swap implementation details (for example, changing the cache provider from Moka to Redis) without altering consumers, just registering different components in the DI container. This facilitated testing (we can inject simulated implementations) and reinforced the dependency inversion principle. On the other hand, the Shaku learning curve and the need to declare modules and components added complexity to the code. It is necessary to keep the container configuration updated as new crates and services are added. Despite this, the choice proved positive: the unified configuration via centralized DI simplified server startup and ensured modules only interact through well-defined interfaces, increasing modularity.
