<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# adapters Module

**Note**: In v0.2.1, adapters have been reorganized into dedicated crates.

**Previous Source**: `src/adapters/`
**New Location**: `crates/mcb-providers/src/` and `crates/mcb-infrastructure/src/adapters/`

## Overview

The adapters layer has been split into two crates in v0.2.1:

1. **mcb-providers** - External service integrations (embedding, vector store, cache, events, language processors)
2. **mcb-infrastructure** - DI composition, config, cross-cutting services

## Migration Mapping

| Old Location | New Location |
| -------------- | -------------- |
| `src/adapters/providers/embedding/` | `crates/mcb-providers/src/embedding/` |
| `src/adapters/providers/vector_store/` | `crates/mcb-providers/src/vector_store/` |
| `src/adapters/hybrid_search/` | `crates/mcb-providers/src/hybrid_search/` |
| `src/adapters/routing/` | `crates/mcb-infrastructure/src/routing/` |

## Related Documentation

- **Providers**: [providers.md](./providers.md) - Provider implementations
- **Infrastructure**: [infrastructure.md](./infrastructure.md) - DI and cross-cutting services
- **Domain**: [domain.md](./domain.md) - Port trait definitions
- **Module Structure**: [module-structure.md](./module-structure.md) - Full architecture

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
