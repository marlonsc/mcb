<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 50
title: Manual Composition Root — dill Removal
status: IMPLEMENTED
created: 2026-02-22
updated: 2026-02-22
related: [23, 24, 29]
supersedes: [29]
superseded_by: []
implementation_status: Complete
---

# ADR 050: Manual Composition Root — dill Removal

## Status

**Implemented** (v0.2.1)

> Supersedes [ADR 029: Hexagonal Architecture with dill](archive/superseded-029-hexagonal-architecture-dill.md).

## Context

ADR-029 introduced the `dill` crate as an IoC container alongside the existing
linkme + Handle pattern. In practice:

1. **`build_catalog()` was never called in production** — `init_app()` in
   `bootstrap.rs` manually wires all services into `AppContext`
2. **dill was the sole dependency requiring nightly Rust** — it uses
   `#![feature(unsize)]` which has no stable Rust timeline
3. **dill usage was trivial** — only `CatalogBuilder::new().add_value().build()`,
   no derives, no macros, no scopes, no validation
4. **CI/CD on stable Rust** was blocked by dill — local dev and CI used
   different toolchains (nightly vs stable), causing constant friction

The actual DI architecture already worked without dill: linkme discovers
providers at compile time, resolvers query the registry, and `init_app()`
wires everything into `AppContext` with explicit field assignment.

## Decision

Remove `dill` and formalize the existing two-layer DI pattern:

### Layer 1: linkme — Compile-Time Provider Discovery

Providers self-register via `#[distributed_slice]` at compile time.
The `impl_registry!` macro in `mcb-domain` generates the slice declaration,
resolver function, and lister function for each provider type.

```rust
// mcb-domain/src/macros/registry.rs — generates per-type registry
impl_registry!(embedding, EmbeddingProvider, EmbeddingConfigContainer);

// mcb-providers/src/embedding/fastembed.rs — self-registration
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static FASTEMBED: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "fastembed",
    description: "Local embedding via ONNX Runtime",
    factory: fastembed_factory,
};
```

**8 distributed slices** across the workspace:

| Slice | Declared in | Implementations |
|-------|-------------|-----------------|
| `EMBEDDING_PROVIDERS` | mcb-domain | FastEmbed, Ollama, OpenAI, VoyageAI, Gemini, Anthropic |
| `VECTOR_STORE_PROVIDERS` | mcb-domain | EdgeVec, Milvus, Qdrant, Pinecone, Encrypted |
| `DATABASE_PROVIDERS` | mcb-domain | SQLite |
| `CACHE_PROVIDERS` | mcb-domain | Moka |
| `LANGUAGE_PROVIDERS` | mcb-domain | TreeSitter |
| `VALIDATION_PROVIDERS` | mcb-domain | mcb-validate |
| `TOOL_DESCRIPTORS` | mcb-server | 9 MCP tools |
| `PROJECT_DETECTORS` | mcb-providers | Cargo, NPM, Maven, Go, Python |

### Layer 2: AppContext — Manual Composition Root

`init_app()` in `bootstrap.rs` is the single composition root. It wires all
services in explicit dependency order:

```text
AppConfig
  ├→ Resolvers (query linkme registry)
  │   └→ Providers (resolved from config)
  │       └→ Handles (RwLock for runtime switching)
  │           └→ AdminServices (API-driven provider swap)
  ├→ DatabaseResolver → SqliteExecutor
  │   └→ Repositories (memory, agent, project, VCS, plan, issue, org)
  └→ Infrastructure (event bus, shutdown, metrics, crypto, highlight)
```

All 49 fields of `AppContext` are explicitly assigned — no reflection,
no container lookup, no runtime resolution.

### Runtime Provider Switching via Handle\<T\>

```rust
// di/handle.rs — generic wrapper
pub struct Handle<T: ?Sized>(RwLock<Arc<T>>);

// AdminService can swap the active provider at runtime:
handle.set(new_provider);  // Atomic swap via RwLock
```

This preserves the hot-swap capability from ADR-029 without needing an
IoC container.

## Consequences

### Positive

1. **Stable Rust everywhere** — CI, local dev, and production use the same toolchain
2. **Zero runtime overhead** — no container lookup, no type erasure, no reflection
3. **Compile-time safety** — Rust's type system enforces the dependency graph
4. **Single composition root** — all wiring visible in one function
5. **Simpler onboarding** — no framework API to learn; just read `init_app()`

### Negative

1. **Manual wiring** — adding a new service requires updating `init_app()` and `AppContext`
2. **No dependency graph validation** — dill could detect circular dependencies (unused)
3. **No visualization** — dill offered graphviz/plantuml export (unused)

### Neutral

1. **linkme unchanged** — provider registration pattern is identical
2. **Handle pattern unchanged** — runtime switching works the same way
3. **Public API unchanged** — `AppContext` accessors remain the same

## Alternatives Considered

### Alternative 1: Keep dill, switch CI to nightly

- **Pros**: No migration effort
- **Cons**: Nightly Rust in production CI adds ABI instability risk, soundness
  bugs, cross-platform complexity, and manual security patching. Major Rust
  users (Amazon, Cloudflare, Discord, Google, Microsoft) all use stable.
- **Rejection**: Risk/maintenance cost not justified for a dependency that
  was never used in production code paths.

### Alternative 2: Replace dill with shaku

- **Pros**: Mature compile-time DI on stable Rust (172K downloads)
- **Cons**: High migration effort, paradigm shift (compile-time vs runtime),
  missing Transaction scope. Would add a new dependency for functionality
  already handled by manual composition.
- **Rejection**: Existing `init_app()` already does everything needed.
  Adding another DI framework would be unnecessary complexity.

### Alternative 3: Wait for dill stable support

- **Pros**: No code changes needed
- **Cons**: `#![feature(unsize)]` stabilization has no timeline. dill TODO
  lists "support stable rust" but no release addresses it. Could be months
  or years.
- **Rejection**: Cannot block CI/toolchain unification on uncertain timeline.

## Implementation Notes

### Changes Made

1. Removed `dill = "0.15"` from workspace `Cargo.toml`
2. Removed `dill = { workspace = true }` from `mcb-infrastructure/Cargo.toml`
3. Deleted `crates/mcb-infrastructure/src/di/catalog.rs` (160 lines)
4. Deleted `crates/mcb-infrastructure/tests/integration/di/catalog_tests.rs` (85 lines)
5. Updated `di/mod.rs` module docs and exports
6. Set `rust-toolchain.toml` to `channel = "stable"`

### Migration Impact

- **Zero production impact** — `build_catalog()` was never called in production
- **4 tests removed** — catalog integration tests (redundant with bootstrap tests)
- **No API changes** — `AppContext` public interface unchanged

## Canonical References

- [PATTERNS.md](../architecture/PATTERNS.md) — Two-Layer DI pattern (normative)
- [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) — System architecture (normative)
- [ADR 023: Inventory to linkme Migration](023-inventory-to-linkme-migration.md)

## References

- [linkme Documentation](https://docs.rs/linkme)
 [ADR 029: Hexagonal Architecture with dill](archive/superseded-029-hexagonal-architecture-dill.md) — Superseded
- [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) — Historical
