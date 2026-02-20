<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# server Module

**Source**: `crates/mcb-server/src/`
**Crate**: `mcb-server`
**Lines of Code**: ~15,800+

**↔ Code ↔ Docs cross-reference**

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-server/src/lib.rs`](../../crates/mcb-server/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-server/src/lib.rs`](../../crates/mcb-server/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-033`](../adr/033-mcp-handler-consolidation.md) · [`ADR-011`](../adr/011-http-transport-request-response-pattern.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The server module implements MCP handlers, admin/web surfaces, transport, hooks, and session management. Handlers are organized into domain-specific subdirectories following the entity-per-module pattern.

## Key Areas

- `handlers/` - MCP tool handlers (domain-split: entities, memory, session, vcs)
- `tools/` - Tool registry and routing
- `transport/` - HTTP/stdio transport and types
- `admin/` - Admin API + web admin routes
- `hooks/` - Hook processing
- `session/` - Session manager/state
- `templates/` - Embedded templates and metadata
- `utils/` - JSON/collection utilities
- `args/` - Consolidated argument types

## Core Root Files

- `auth.rs`, `args.rs`, `builder.rs`, `constants.rs`, `error_mapping.rs`, `formatter.rs`, `init.rs`, `mcp_server.rs`, `lib.rs`

## File Structure

```text
crates/mcb-server/src/
├── admin/
│   └── web/
├── args/
│   └── consolidated.rs
├── handlers/
│   ├── entities/           # Entity CRUD handlers
│   │   ├── common.rs       -> [crates/mcb-server/src/handlers/entities/common.rs]
│   │   ├── issue.rs        -> [crates/mcb-server/src/handlers/entities/issue.rs]
│   │   ├── org.rs          -> [crates/mcb-server/src/handlers/entities/org.rs]
│   │   ├── plan.rs         -> [crates/mcb-server/src/handlers/entities/plan.rs]
│   │   ├── vcs.rs          -> [crates/mcb-server/src/handlers/entities/vcs.rs]
│   │   └── mod.rs
│   ├── memory/             # Memory tool handlers
│   │   ├── common.rs       -> [crates/mcb-server/src/handlers/memory/common.rs]
│   │   ├── execution.rs    -> [crates/mcb-server/src/handlers/memory/execution.rs]
│   │   ├── handler.rs      -> [crates/mcb-server/src/handlers/memory/handler.rs]
│   │   ├── inject.rs       -> [crates/mcb-server/src/handlers/memory/inject.rs]
│   │   ├── list_timeline.rs -> [crates/mcb-server/src/handlers/memory/list_timeline.rs]
│   │   ├── observation.rs  -> [crates/mcb-server/src/handlers/memory/observation.rs]
│   │   ├── quality_gate.rs -> [crates/mcb-server/src/handlers/memory/quality_gate.rs]
│   │   ├── session.rs      -> [crates/mcb-server/src/handlers/memory/session.rs]
│   │   └── mod.rs
│   ├── session/            # Session tool handlers
│   │   ├── common.rs
│   │   ├── create.rs
│   │   ├── get.rs
│   │   ├── handler.rs
│   │   ├── list.rs
│   │   ├── summarize.rs
│   │   ├── update.rs
│   │   └── mod.rs
│   ├── vcs/                # VCS tool handlers
│   │   ├── browse.rs
│   │   ├── common.rs
│   │   ├── handler.rs
│   │   ├── list_branches.rs
│   │   ├── read_file.rs
│   │   ├── sync.rs
│   │   └── mod.rs
│   └── mod.rs
├── hooks/
├── session/
├── templates/
├── tools/
├── transport/
├── utils/
├── auth.rs
├── builder.rs
├── constants.rs
├── error_mapping.rs
├── formatter.rs
├── init.rs
├── mcp_server.rs
└── lib.rs
```

## Testing Strategy

- `handlers/` unit tests for each tool
- Integration tests in `tests/` for full MCP protocol exchange
- Mocking of application services via traits

## Related Documentation

- **MCP Protocol**: [MCP Specification](https://modelcontextprotocol.io)
- **Tool Specification**: [MCP_TOOLS.md](../MCP_TOOLS.md)
- **Admin API**: [admin.md](./admin.md)
- **Transport**: [stdio, http]
- **Handlers**: Internal mapping to use cases in `mcb-application`

---

### Updated 2026-02-20 - Corrected stale structure; added Source links to handlers; verified against actual `mcb-server` layout (v0.2.1)
