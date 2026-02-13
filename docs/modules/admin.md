<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# admin Module

**Source**: `crates/mcb-server/src/admin/`
**Crate**: `mcb-server`
**Files**: 22
**Lines of Code**: ~6,456

## Overview

The admin module provides configuration, lifecycle, browse, and web admin surfaces for server operations.

### Key Components

### API and Lifecycle

- `api.rs` - Admin API entrypoints
- `handlers.rs` - Core admin handlers
- `lifecycle_handlers.rs` - Lifecycle actions
- `auth.rs` - Admin auth integration

### Configuration and Registry

- `config.rs` - Admin configuration models
- `config_handlers.rs` - Config endpoints
- `registry.rs` - Entity/route registry
- `crud_adapter.rs` - Generic CRUD adapter

### Transport and Streaming

- `routes.rs` - Route setup
- `sse.rs` - Server-sent events
- `propagation.rs` - Change propagation wiring

### Web Admin Surface

- `web/handlers.rs` - HTML route handlers
- `web/entity_handlers.rs` - Entity pages/actions
- `web/lov_handlers.rs` - LOV endpoints
- `web/filter.rs` - Filtering helpers
- `web/view_model.rs` - UI view models
- `web/router.rs` - Web admin router
- `web/templates/` - Shared JS/CSS assets

## File Structure

```text
crates/mcb-server/src/admin/
├── api.rs
├── auth.rs
├── browse_handlers.rs
├── config.rs
├── config_handlers.rs
├── crud_adapter.rs
├── handlers.rs
├── lifecycle_handlers.rs
├── models.rs
├── propagation.rs
├── registry.rs
├── routes.rs
├── sse.rs
├── mod.rs
└── web/
    ├── entity_handlers.rs
    ├── filter.rs
    ├── handlers.rs
    ├── helpers.rs
    ├── lov_handlers.rs
    ├── router.rs
    ├── view_model.rs
    └── templates/
```

## Cross-References

- **Server**: [server.md](./server.md)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
