<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 54
title: mcb-utils as Innermost Layer 0 Crate
status: ACCEPTED
created: 2026-03-02
updated: 2026-03-02
related: [13, 23, 50]
supersedes: []
superseded_by: []
implementation_status: Accepted
---

# ADR 054: mcb-utils as Innermost Layer 0 Crate

## Status

**Accepted** (v0.3.0)

> Establishes Clean Architecture boundary for shared utilities.

## Context

The `mcb-domain` crate currently contains both:

1. **Abstract domain layer** — traits, ports, entities, value objects, and the provider registry
2. **Utility implementations** — shared helper code in `utils/` modules and project-wide constants

This violates Clean Architecture principles by mixing pure domain concepts (Layer 1) with shared helper code that has no domain knowledge. The utilities (`truncate_string`, `parse_version`, timing helpers, etc.) are used across all crates but carry no semantic meaning about the domain.

Additionally, `mcb-domain` defines error types used throughout the workspace. These errors often wrap utility-level failures (IO errors, parsing errors) but must convert them into domain errors. Without a dedicated utility layer, this conversion logic leaks implementation concerns into the domain layer.

## Decision

Extract all shared utilities and constants into a new `mcb-utils` crate positioned as **Layer 0** (innermost) in the Clean Architecture hierarchy.

### New Layer Structure

```text
Layer 0: mcb-utils          (innermost — no domain knowledge)
    ↓
Layer 1: mcb-domain         (abstract: traits, ports, entities, registry)
    ↓
Layer 2: mcb-infrastructure (use cases, DI, config, cache)
    ↓
Layer 3: mcb-providers      (embedding, vector store implementations)
    ↓
Layer 4: mcb-validate       (architecture validation)
    ↓
Layer 5: mcb-server         (MCP protocol, transport)
    ↓
Layer 6: mcb                (binary, composition root)
```

### mcb-utils Responsibilities

- **Zero domain knowledge** — no references to providers, entities, or business logic
- **Shared utilities** — string manipulation, parsing, timing, collections helpers
- **Constants** — project-wide constants (timeouts, limits, magic numbers)
- **Error types** — utility-level errors via `thiserror` for use by all layers

### Error Handling Boundary

```rust
// mcb-utils/src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum UtilsError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// mcb-domain/src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("utility failure: {0}")]
    Utils(#[from] mcb_utils::UtilsError),
    // ... domain errors
}
```

### Workspace Structure

```text
crates/
├── mcb/                    # Binary crate (Layer 6)
├── mcb-utils/              # NEW: Utilities layer (Layer 0)
├── mcb-domain/             # Domain layer (Layer 1)
├── mcb-infrastructure/     # Infrastructure layer (Layer 2)
├── mcb-providers/          # Provider implementations (Layer 3)
├── mcb-validate/           # Architecture validation (Layer 4)
└── mcb-server/             # MCP server (Layer 5)
```

### Dependencies

```text
mcb-utils:      (no mcb-* deps)
mcb-domain:     mcb-utils
mcb-infrastructure: mcb-domain, mcb-utils
mcb-providers:  mcb-domain, mcb-utils
mcb-validate:   mcb-domain, mcb-utils
mcb-server:     mcb-infrastructure, mcb-domain, mcb-utils
mcb:            all of the above
```

## Consequences

### Positive

1. **Strict Clean Architecture** — Layer 0 has zero domain knowledge; Layer 1 is pure abstraction
2. **Reusable utilities** — Helpers can be used by any crate without pulling in domain concepts
3. **Clear error boundaries** — Utility errors convert to domain errors via `From` impls
4. **Simpler testing** — Utility functions testable without domain setup
5. **Reduced coupling** — Domain layer no longer contains non-domain code
6. **7-crate workspace** — Explicit architectural layers make dependency flow obvious

### Negative

1. **New crate overhead** — Additional `Cargo.toml`, module structure, and maintenance
2. **Migration effort** — Move existing `utils/` modules from `mcb-domain` to `mcb-utils`
3. **Import changes** — Existing code must update imports from `mcb_domain::utils` to `mcb_utils`
4. **Version coordination** — One more crate to version and publish

### Neutral

1. **linkme unchanged** — Provider registration remains in `mcb-domain`
2. **DI pattern unchanged** — `AppContext` composition root works the same way
3. **Public API evolution** — `mcb_utils` exports become part of the stable utility API

## Architecture Validation Updates

The architecture validation rules must recognize the new 7-crate structure:

- **Rule update**: Layer 0 (`mcb-utils`) has no inward dependencies (no `use mcb_`)
- **Rule update**: Layer 1 (`mcb-domain`) may only depend on Layer 0
- **Rule unchanged**: Layers 2-6 maintain existing dependency constraints

## References

- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) — Original layer separation
- [ADR 023: Inventory to Linkme Migration](023-inventory-to-linkme-migration.md) — Provider registration pattern
- [ADR 050: Manual Composition Root — dill Removal](050-manual-composition-root-dill-removal.md) — Current DI architecture
- [Clean Architecture](../architecture/CLEAN_ARCHITECTURE.md) — Layer rules and dependency flow
