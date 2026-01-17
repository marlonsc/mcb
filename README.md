# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.1.1-blue)](https://github.com/marlonsc/mcp-context-browser/releases)

**High-performance MCP server for persistent multimodal context storage** - Provides persistent, multimodal context storage and access for AI agents. Allows AI applications (like Claude, Cursor, VS Code, etc.) to consult and update context information in real-time through a standardized API.

## Overview

MCP Context Browser is a Model Context Protocol (MCP) server that provides persistent multimodal context storage and access for AI agents. It allows AI applications (like Claude, Cursor, VS Code, etc.) to consult and update context information in real-time through a standardized API. The project's objective is to offer an extensible enterprise-grade architecture, with support for multiple data providers, caching, and an administrative panel for monitoring.

### Main Features

-   **Modular Architecture**: Built in Rust with multiple crates, separating functionalities (core, providers, event bus, etc.) and facilitating expansion.
-   **Dependency Injection**: Uses Shaku DI to manage components flexibly, allowing implementation swapping (e.g., cache backend, messaging) via configuration.
-   **Unified Providers**: Integrated support for various context providers under a common interface (local memory, files, external APIs - easily extensible).
-   **Event Bus**: Internal asynchronous events mechanism for decoupled communication between components, using Tokio by default (local) or NATS JetStream for distributed events.
-   **Configurable Cache**: Improves performance with in-memory cache (Moka) and optional distributed cache with Redis, controlled via configuration.
-   **Admin UI with SSE**: Includes a web administrative panel (Server-Sent Events) for real-time server activity monitoring (events, logs, provider status).

## Architecture

The MCP Context Browser architecture follows low coupling and high cohesion principles between components:

### Modular Crates

The project is divided into crates, for example: mcp_context_browser_core, mcp_context_browser_providers, mcp_context_browser_eventbus, etc. Each crate concentrates a set of responsibilities. This makes the codebase more manageable and allows crate reuse in other MCP projects.

### Service Manager & DI

The core crate initializes the ServiceManager, which loads all services (providers, cache, event bus, etc.) using the Shaku dependency container. Dependencies are injected as declared in each crate's Shaku modules, ensuring each component receives the correct implementations according to active configuration.

### EventBus

It is a central part of the architecture - offers an internal event broker. Modules publish events (e.g., "Context X updated") in the EventBus and other modules can subscribe to react. With the default implementation (Tokio), communication is internal to the process. With the NATS implementation activated, events are also sent to a NATS server, synchronizing multiple MCP Context Browser instances.

### Cache

All context data accesses can pass through a unified cache layer. By default, Moka (local in-memory cache) is used. If configured, Redis is used for shared cache. The choice is abstract for consumer modules - they use the cache interface without worrying about the backend.

### Providers

They are context data suppliers (for example, a "conversation memory" provider, a provider that searches documentation files, etc.). Each provider implements the common trait and is registered via DI. They can emit events (via EventBus) and use cache as needed, following defined policies.

### Coordinated Shutdown

The ShutdownCoordinator ensures that when shutting down the server, all providers and services terminate orderly (for example, waiting for event flushing or unloading volatile data in storage, if needed). This avoids data loss or state corruption in abrupt shutdown scenarios.

For historical details of design decisions, consult the complete ADRs in the docs/architecture/ folder (or Architecture section in the documentation site):

ADR 001 – Modular Crates Architecture

ADR 002 – Dependency Injection with Shaku

ADR 003 – Unified Provider Architecture

ADR 004 – Event Bus (Local and Distributed)

ADR 005 – Context Cache Support (Moka/Redis)

(The ADRs summarize motivations, decisions and consequences of each architectural choice.)

## Usage Instructions

### Requirements

Rust 1.65+ (edition 2021 or superior) and Tokio runtime. For optional features: a running NATS server (if EventBus distributed is enabled) and/or a Redis server (if external cache is enabled).

### Build and Execution

Clone the repository and navigate to the project directory.

Execute `cargo build --release` to compile the server in release mode.

Execute `cargo run --bin mcp-context-browser` to run the server (or use the binary generated in target/release). By default, the server will read the configuration file config.toml in the current directory (or equivalent environment variables).

### MCP Integration

Once running, the Context Browser exposes MCP endpoints via HTTP (host and port configurable, default 127.0.0.1:8080). Client applications (for example, agents in VS Code or another IDE with MCP support) can connect to this server. Consult the MCP documentation for protocol details; in summary, you will be able to send standardized HTTP requests to store, consult or delete context items. The server handles these requests by routing them to appropriate providers, using cache and publishing events as configured.

### Quick Example

Via terminal, using curl, we can add and retrieve a context item:

```bash
# Add a context item (example JSON in body)
curl -X POST http://localhost:8080/context -d '{"key": "note1", "value": "This is a test note"}'

# Retrieve the added context item
curl http://localhost:8080/context/note1
```

The responses will follow the MCP format (normally JSON with fields like value, timestamp, etc.).

## Configuration

The MCP Context Browser behavior is extensively configurable via TOML file (by default config.toml). Below we exemplify the main configurations, including cache and EventBus options:

### Default Configuration (Moka + Tokio)

In this configuration, we use local in-memory cache and internal Tokio EventBus.

```toml
[server]
host = "127.0.0.1"
port = 8080

[event_bus]
# Event bus mode: "tokio" for internal, "nats" for distributed
provider = "tokio"
# NATS configurations (ignored if provider=tokio)
nats_url = "nats://localhost:4222"
nats_subject = "mcp_context_browser.events"

[cache]
# Cache mode: "moka" for internal in-memory, "redis" for external Redis
provider = "moka"
# Redis configurations (ignored if provider=moka)
redis_url = "redis://localhost:6379"
redis_db = 0
default_ttl_secs = 300  # Default cache expiration time in seconds
```

### Alternative Configuration (Redis + NATS)

Example to enable Redis as cache and NATS as event bus:

```toml
[server]
host = "0.0.0.0"
port = 8080

[event_bus]
provider = "nats"
nats_url = "nats://nats.myserver.local:4222"
nats_subject = "mcp_context.events"
# (Other NATS options like credentials can be included as needed)

[cache]
provider = "redis"
redis_url = "redis://my-redis.local:6379"
redis_db = 5
default_ttl_secs = 600
```

Besides these, the configuration file can also adjust logs, debug level, activate/deactivate specific providers, etc. All supported parameters are documented in docs/configuration.md (placeholder for detailed configuration documentation).

Note: If you don't want to use a configuration file, equivalent environment variables can be defined (the system supports both forms). For example, `EVENT_BUS__PROVIDER=nats` could be used instead of the field in TOML.

## Testing

The MCP Context Browser includes a comprehensive automated test suite. To run them, execute:

```bash
cargo test
```

The tests cover provider functionality, context operations (CRUD), cache and EventBus behavior. Unit tests mainly use the Tokio EventBus and isolated in-memory cache for determinism. There are also integration tests that simulate different configurations - for example, verifying that when switching to Redis or NATS (if services are accessible during testing), the system behaves as expected.

To run tests that involve NATS/Redis, it is recommended to have instances of these services running locally. Alternatively, you can skip or mock these tests by defining flags/variables (see testing documentation in docs/developer/testing.md). All tests should pass in both scenarios (default and with external resources) to ensure modularity works correctly.

## Admin Panel (SSE)

The Context Browser offers a web administration panel that can be used to monitor the server in real-time. This panel uses Server-Sent Events (SSE) to update information dynamically in the browser, allowing activity tracking without needing to reload the page.

### Access

By default, after starting the server, the panel is accessible at http://localhost:8080/admin (adjust host/port according to your configuration). When accessing this URL via browser, you will see a simple interface showing server statistics, list of active providers, last activity of each one, and a stream of recent events.

The displayed events include context addition/update, cache hits/misses, and any runtime warnings/errors. Thanks to SSE, new events appear in real-time on the page as they are emitted by the EventBus.

The panel also allows triggering a metrics collection or viewing the current state of each provider (for example, cache size, Redis or NATS connection status if in use).

To use the admin panel, open the URL in a modern browser. Ensure that SSE is not blocked by firewall or proxy, as it maintains an open HTTP connection for streaming. Note: The admin panel has no write functionalities - it is read-only/monitoring, ensuring security (any context modification must be done via authenticated MCP endpoints, as per server security implementation).

## Security and Authentication

(If applicable, describe authentication mechanisms, access control, etc. If not, plan to include in future version.) By default, the server can be configured to require an API key or token to accept MCP commands, in order to avoid unauthorized accesses. See docs/security.md for recommended secure deployment practices.

## Contributing

Contributions are welcome! Feel free to open issues and pull requests. Make sure to follow the contribution guide in CONTRIBUTING.md and maintain consistent code style (rustfmt, clippy) and updated documentation (including ADRs when necessary) when proposing significant architecture changes.

## License

MIT Licensed - Open source and free for commercial and personal use.
