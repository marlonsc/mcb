# MCB Architecture

## Clean Architecture Layers

Dependency flow is strictly inward:
```
server → infrastructure → providers → domain → utils
```

### Layer Rules
| Crate | MUST NOT depend on | Allowed Dependencies |
|-------|-------------------|---------------------|
| mcb-utils | Any internal crate | None (pure utilities) |
| mcb-domain | Any internal crate except mcb-utils | mcb-utils only |
| mcb-providers | mcb-infrastructure, mcb-server | mcb-domain, mcb-utils |
| mcb-infrastructure | mcb-server | mcb-domain, mcb-providers, mcb-utils |
| mcb-server | None | mcb-infrastructure, mcb-utils |

## Dependency Injection
- **Compile-time registration**: linkme `#[distributed_slice]` for provider discovery
- **Runtime switching**: RwLock-wrapped handles in AppContext
- **Composition root**: `mcb-infrastructure/src/di/bootstrap.rs`

## Provider Pattern
All external integrations implement domain port traits:
- Location: `mcb-domain/src/ports/providers/`
- Examples: `EmbeddingProvider`, `VectorStoreProvider`, `CacheProvider`
- Registration: linkme distributed slice in provider module

## Key Architectural Decisions
- **ADR-050**: Manual composition root (no framework DI)
- **ADR-029**: All port traits in mcb-domain (single source of truth)
- **ADR-013**: Clean Architecture crate separation
- **ADR-054/055**: Constants consolidated in mcb-utils

## Validation
Run `make validate` to enforce architecture rules via mcb-validate:
- Phase 1: Linter checks (Clippy)
- Phase 2: AST pattern queries (Tree-sitter)
- Phase 3: Rule engine validation
- Phase 4: Metrics analysis
- Phase 5: Duplication detection
- Phase 6: Architecture validation
