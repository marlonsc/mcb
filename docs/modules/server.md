## Project links

- `docs/context/technical-patterns.md` (MCP tool expectations).  
- `docs/context/project-state.md` + `.planning/STATE.md` (Phase 6 progress, Hybrid Search plan).  
- `docs/context/domain-concepts.md`, `docs/context/integrations.md`, and `docs/developer/ROADMAP.md` (v0.2.0 vision).  
- `docs/operations/CHANGELOG.md` for metrics that change when server components are added.

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

-   `index_codebase.rs` - Index repository tool
-   `search_code.rs` - Semantic search tool
-   `get_indexing_status.rs` - Status query tool
-   `clear_index.rs` - Index clearing tool

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
│   ├── index_codebase.rs     # Index tool
│   ├── search_code.rs        # Search tool
│   ├── get_indexing_status.rs # Status tool
│   ├── clear_index.rs        # Clear tool
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

- **Phase context**: Keep HTTP transport work in lockstep with `docs/context/project-state.md`/`.planning/STATE.md` so Phase 6 Hybrid Search (06-02) and release `release/v0.1.5` remain synchronized.  
- **Architecture guidance**: `docs/architecture/ARCHITECTURE.md` describes the layering that this module sits atop, while `docs/context/technical-patterns.md` captures patterns for MCP tooling and provider registration.  
- **Roadmap signals**: Refer to `docs/developer/ROADMAP.md` for the v0.2.0 objectives (git-aware indexing, session memory, advanced browser) and `docs/context/domain-concepts.md` for domain usage.  
- **Operational metrics**: When tools or admin handlers change, reflect the counts in `docs/operations/CHANGELOG.md` and `docs/operations/CI_OPTIMIZATION_VALIDATION.md` so the documentation and tests stay aligned.

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.1.2)*
