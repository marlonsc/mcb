<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# server Module

**Source**: `crates/mcb-server/src/`
**Crate**: `mcb-server`
**Lines of Code**: ~15,800+

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
│   │   ├── common.rs
│   │   ├── issue.rs
│   │   ├── org.rs
│   │   ├── plan.rs
│   │   ├── vcs.rs
│   │   └── mod.rs
│   ├── memory/             # Memory tool handlers
│   │   ├── common.rs
│   │   ├── execution.rs
│   │   ├── handler.rs
│   │   ├── inject.rs
│   │   ├── list_timeline.rs
│   │   ├── observation.rs
│   │   ├── quality_gate.rs
│   │   ├── session.rs
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
│   │   ├── analyze_impact.rs
│   │   ├── compare_branches.rs
│   │   ├── handler.rs
│   │   ├── index_repo.rs
│   │   ├── list_repos.rs
│   │   ├── responses.rs
│   │   ├── search_branch.rs
│   │   └── mod.rs
│   ├── agent.rs
│   ├── helpers.rs
│   ├── index.rs
│   ├── macros.rs
│   ├── project.rs
│   ├── search.rs
│   ├── validate.rs
│   └── mod.rs
├── hooks/
├── session/
├── templates/
├── tools/
├── transport/
├── utils/
├── args.rs
├── auth.rs
├── builder.rs
├── constants.rs
├── error_mapping.rs
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

### Updated 2026-02-14 - Added handler subdirectories (entities, memory, session, vcs), args/, error_mapping.rs (v0.2.1)
