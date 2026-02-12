# server Module

**Source**: `crates/mcb-server/src/`
**Crate**: `mcb-server`
**Files**: 94
**Lines of Code**: ~15,812

## Overview

The server module implements MCP handlers, admin/web surfaces, transport, hooks, and session management.

## Key Areas

- `handlers/` - MCP tool handlers
- `tools/` - Tool registry and routing
- `transport/` - HTTP/stdio transport and types
- `admin/` - Admin API + web admin routes
- `hooks/` - Hook processing
- `session/` - Session manager/state
- `templates/` - Embedded templates and metadata
- `utils/` - JSON/collection utilities

## Core Root Files

- `auth.rs`, `args.rs`, `builder.rs`, `constants.rs`, `formatter.rs`, `init.rs`, `mcp_server.rs`, `lib.rs`

## File Structure

```text
crates/mcb-server/src/
├── admin/
│   └── web/
├── handlers/
├── tools/
├── transport/
├── hooks/
├── session/
├── templates/
├── utils/
├── auth.rs
├── args.rs
├── builder.rs
├── constants.rs
├── formatter.rs
├── init.rs
├── mcp_server.rs
└── lib.rs
```

## Testing

Server tests are in `crates/mcb-server/tests/`.

## Cross-References

- **Admin**: [admin.md](./admin.md)
- **Domain**: [domain.md](./domain.md)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)*
