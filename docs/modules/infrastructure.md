<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# infrastructure Module

**Source**: `crates/mcb-infrastructure/src/`
**Crate**: `mcb-infrastructure`
**Files**: 73
**Lines of Code**: ~7,860

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-infrastructure/src/lib.rs`](../../crates/mcb-infrastructure/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-infrastructure/src/lib.rs`](../../crates/mcb-infrastructure/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-029`](../adr/029-hexagonal-architecture-dill.md) · [`ADR-023`](../adr/023-inventory-to-linkme-migration.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The infrastructure module contains DI bootstrap, configuration, routing, project wiring, and shared technical services.

## Top-Level Areas

- [`di/`](../../crates/mcb-infrastructure/src/di/) - Container/bootstrap and module wiring
- [`config/`](../../crates/mcb-infrastructure/src/config/) - Configuration parsing and typed config
- [`constants/`](../../crates/mcb-infrastructure/src/constants/) - Shared constants
- [`project/`](../../crates/mcb-infrastructure/src/project/) - Project-oriented helpers
- [`services/`](../../crates/mcb-infrastructure/src/services/) - Infrastructure services
- [`validation/`](../../crates/mcb-infrastructure/src/validation/) - Validation support
- [`routing/`](../../crates/mcb-infrastructure/src/routing/) - Routing and dispatch helpers
- [`utils/`](../../crates/mcb-infrastructure/src/utils/) - Utility helpers
- [`cache/`](../../crates/mcb-infrastructure/src/cache/) - Caching infrastructure
- [`crypto/`](../../crates/mcb-infrastructure/src/crypto/) - Cryptographic utilities
- [`infrastructure/`](../../crates/mcb-infrastructure/src/infrastructure/) - Cross-cutting infrastructure modules
- [`error_ext.rs`](../../crates/mcb-infrastructure/src/error_ext.rs) - Error extension traits
- [`health.rs`](../../crates/mcb-infrastructure/src/health.rs) - Health check infrastructure
- [`logging/`](../../crates/mcb-infrastructure/src/logging/) - Logging configuration (tracing)
- [`macros.rs`](../../crates/mcb-infrastructure/src/macros.rs) - Infrastructure macros

## File Structure

```text
crates/mcb-infrastructure/src/
├── cache/
├── config/
├── constants/
├── crypto/
├── di/
├── infrastructure/
├── project/
├── routing/
├── services/
├── utils/
├── validation/
├── error_ext.rs
├── health.rs
├── logging/
├── macros.rs
└── lib.rs
```

## Testing

Infrastructure tests are in `crates/mcb-infrastructure/tests/`.

## Cross-References

- **Domain**: [domain.md](./domain.md)
- **Server**: [server.md](./server.md)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-15 - Fixed logging.rs to logging/, added macros.rs (v0.2.1)
