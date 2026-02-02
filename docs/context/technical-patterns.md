# Technical Patterns Context

**Last updated:** 2026-02-02
**Source:** `AGENTS.md` (architecture guidance) and `README.md` (product overview)

## Overview
The MCP Context Browser keeps its codebase aligned with Clean Architecture layers and distributed-provider discovery so the runtime stays lean while each crate can evolve independently.

## Key Patterns

### Clean Architecture crate layering
**Used in:** `AGENTS.md` "Architecture (9 Crates)"
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
**Used in:** `AGENTS.md` "Provider Registration (linkme)"
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
**Used in:** `AGENTS.md` "Traits & DI" and "Error Handling"
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

## Related Context
- `docs/architecture/ARCHITECTURE.md` (system-level architecture description)
- `docs/adr/023-inventory-to-linkme-migration.md` (provider migration rationale)
