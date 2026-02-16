<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# infrastructure Module

**Source**: `crates/mcb-infrastructure/src/`
**Crate**: `mcb-infrastructure`
**Files**: 73
**Lines of Code**: ~7,860

## Overview

The infrastructure module contains DI bootstrap, configuration, routing, project wiring, and shared technical services.

## Top-Level Areas

- `di/` - Container/bootstrap and module wiring
- `config/` - Configuration parsing and typed config
- `constants/` - Shared constants
- `project/` - Project-oriented helpers
- `services/` - Infrastructure services
- `validation/` - Validation support
- `routing/` - Routing and dispatch helpers
- `utils/` - Utility helpers
- `cache/` - Caching infrastructure
- `crypto/` - Cryptographic utilities
- `infrastructure/` - Cross-cutting infrastructure modules
- `error_ext.rs` - Error extension traits
- `health.rs` - Health check infrastructure
- `logging/` - Logging configuration (tracing)
- `macros.rs` - Infrastructure macros

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
