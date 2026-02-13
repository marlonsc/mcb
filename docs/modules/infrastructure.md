<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
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
- `cache/`, `crypto/`, `infrastructure/` - Cross-cutting infrastructure modules

## File Structure

```text
crates/mcb-infrastructure/src/
├── di/
├── config/
├── constants/
├── project/
├── services/
├── validation/
├── routing/
├── utils/
├── cache/
├── crypto/
├── infrastructure/
└── lib.rs
```

## Testing

Infrastructure tests are in `crates/mcb-infrastructure/tests/`.

## Cross-References

- **Domain**: [domain.md](./domain.md)
- **Server**: [server.md](./server.md)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects dill IoC and current crate structure (v0.2.1)
