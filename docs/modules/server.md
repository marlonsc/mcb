## Project links

-   `docs/architecture/ARCHITECTURE.md` (MCP tool expectations and architecture).
-   `docs/modules/domain.md` and `docs/developer/ROADMAP.md` (v0.2.1 vision).
-   `docs/operations/CHANGELOG.md` for metrics that change when server components are added.

# server Module

**Source**: `crates/mcb-server/src/`
**Crate**: `mcb-server`
**Files**: 20+
**Lines of Code**: ~4,500

## Overview

The server module provides the MCP protocol implementation and HTTP transport layer. It includes tool handlers, admin API, authentication, and server initialization.

## Key Components

### MCP Server (`mcp_server.rs`)

Core MCP protocol server implementation with JSON-RPC handling.

### Tool Handlers (`handlers/`)

MCP tool implementations:

-   `index` - Index operations (start/status/clear)
-   `search` - Unified search (code/memory)
-   `validate` - Validation + complexity analysis
-   `memory` - Memory storage, retrieval, timeline, inject
-   `session` - Session lifecycle + summaries
-   `agent` - Agent activity logging
-   `project` - Project workflow operations
-   `vcs` - Repository operations

### Admin API (`admin/`)

Administrative endpoints:

-   `handlers.rs` - Health check, metrics, shutdown handlers
-   `config_handlers.rs` - Configuration management handlers
-   `routes.rs` - Axum router configuration
-   `models.rs` - Request/response types
-   `service.rs` - Admin service orchestration

### Transport (`transport/`)

HTTP transport layer:

-   `http.rs` - HTTP server setup
-   `session.rs` - Session management
-   `config.rs` - Transport configuration
-   `versioning.rs` - API versioning

### Authentication (`auth.rs`)

JWT-based authentication and authorization.

### Initialization (`init.rs`)

Server startup and DI container bootstrapping.

## File Structure

```text
crates/mcb-server/src/
├── admin/
│   ├── handlers.rs           # Admin endpoint handlers
│   ├── config_handlers.rs    # Config management
│   ├── routes.rs             # Router setup
│   ├── models.rs             # Request/response types
│   ├── service.rs            # Admin service
│   └── mod.rs
├── handlers/
│   ├── agent.rs
│   ├── index.rs
│   ├── memory.rs
│   ├── project.rs
│   ├── search.rs
│   ├── session.rs
│   ├── validate.rs
│   ├── vcs.rs
│   └── mod.rs
├── transport/
│   ├── http.rs               # HTTP server
│   ├── session.rs            # Sessions
│   ├── config.rs             # Transport config
│   └── versioning.rs         # API versions
├── tools/
│   └── mod.rs                # Tool registry
├── args.rs                   # CLI arguments
├── auth.rs                   # Authentication
├── builder.rs                # Server builder
├── constants.rs              # Server constants
├── formatter.rs              # Output formatting
├── init.rs                   # Initialization
├── mcp_server.rs             # MCP protocol
├── main.rs                   # Entry point
└── lib.rs                    # Crate root
```

## Key Exports

```rust
// Server
pub use mcp_server::McpServer;
pub use builder::McpServerBuilder;
pub use init::run_server;

// Admin
pub use admin::{AdminService, HealthResponse};
```

## Testing

Server tests are located in `crates/mcb-server/tests/`.

## Project Alignment

-   **Architecture guidance**: `docs/architecture/ARCHITECTURE.md` describes the layering that this module sits atop and captures patterns for MCP tooling and provider registration.
-   **Roadmap signals**: Refer to `docs/developer/ROADMAP.md` for the v0.2.1 objectives (git-aware indexing, session memory, advanced browser) and `docs/modules/domain.md` for domain usage.
-   **Operational metrics**: When tools or admin handlers change, reflect the counts in `docs/operations/CHANGELOG.md` and `docs/operations/CI_OPTIMIZATION_VALIDATION.md` so the documentation and tests stay aligned.

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.2.1)*
