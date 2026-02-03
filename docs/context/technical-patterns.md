# Technical Patterns Context

**Last updated:** 2026-02-02
**Source:** `README.md` (project overview) and `docs/architecture/ARCHITECTURE.md` (system architecture)

## Overview

The MCP Context Browser keeps its codebase aligned with Clean Architecture layers and distributed-provider discovery so the runtime stays lean while each crate can evolve independently.

## Key Patterns

### Clean Architecture crate layering

**Used in:** `README.md` "Architecture" summary and `docs/architecture/ARCHITECTURE.md`

```
crates/
├── mcb/             # Facade re-exporting the public API
├── mcb-domain/      # Entities, ports, and errors
├── mcb-application/ # Use cases, services, linkme registry
├── mcb-providers/   # Embedding/vector-store implementations
├── mcb-infrastructure/# DI, config, crypto, logging
├── mcb-server/      # MCP protocol handlers
├── mcb-validate/    # Architecture validation tooling
├── mcb-language-support/ # Language detection
└── mcb-ast-utils/   # AST parsing helpers
```

**When to use:** Keep new functionality within this layered structure and respect the dependency direction: `mcb-server → mcb-infrastructure → mcb-application → mcb-domain` with `mcb-providers` being consumed but never depending upstream.

### Linkme provider registration

**Used in:** `docs/architecture/ARCHITECTURE.md` provider registration section

```rust
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static NULL_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "null",
    description: "Null provider for testing",
    factory: null_embedding_factory,
};
```

**When to use:** Any new embedding/vector-store provider must follow this registration so the distributed slice auto-discovers it at compile time and no manual wiring is required.

### Async traits and error factory methods

**Used in:** `docs/architecture/ARCHITECTURE.md` and `mcb-domain` module docs covering errors and ports

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
}

impl Error {
    pub fn io<S: Into<String>>(message: S) -> Self { ... }
}
```

**When to use:** Implement providers with `#[async_trait]`, return workspace `Result` types, and create helper constructors for custom errors instead of `panic!`ing or using `unwrap()`.

## Project-phase Patterns

### Hybrid Search (Phase 6)

**Used in:** `.planning/STATE.md` and `docs/developer/ROADMAP.md` latest sections

-   Phase 6 "Memory Search" is in progress with plan 06-02 (Hybrid Search Implementation) next in line.
-   The project batches FTS5 infrastructure work (triggers, deduplication by SHA256) before layering vector search on top again.
**When to use:** Align any new feature with the release branch `release/v0.1.5` and cross-check against the plan checklist so Phase 6 artifacts stay coordinated.

### Metrics awareness

**Used in:** `.planning/PROJECT.md`

-   Track provider counts (7), vector stores (8), languages (14), and 1805+ tests as part of documentation metrics injection.
-   Pre-commit hooks automatically refresh badges and metrics in `docs/user-guide/README.md`, `docs/developer/ROADMAP.md`, and `docs/operations/CHANGELOG.md`.
**When to use:** Update metrics data whenever capability counts change so the `docs/generated/METRICS.md` stays accurate and the documentation reflects the live project state.

## Related Context

-   `docs/architecture/ARCHITECTURE.md` (system-level architecture description)
-   `docs/adr/023-inventory-to-linkme-migration.md` (provider migration rationale)
