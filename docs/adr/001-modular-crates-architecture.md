<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
<!-- markdownlint-disable MD025 -->
adr: 1
title: Modular Crates Architecture
status: IMPLEMENTED
created:
updated: 2026-02-05
related: [2, 3, 4, 5]
supersedes: []
superseded_by: []
implementation_status: Complete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

## ADR 001: Modular Crates Architecture

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.

## Status

**Implemented** (v0.1.1)

> Fully implemented with Clean Architecture + linkme + Handle DI across 6 crates.

## Context

Initially, the Memory Context Browser had a monolithic architecture. As the
project grew, the need for better code organization, separation of concerns, and
component reusability emerged. We evaluated adopting a modular architecture by
dividing the system into multiple Rust crates, each responsible for a specific
domain or functionality (e.g., core server crate, context providers crate,
inter-module communication crate, etc.). We also considered how to manage the
orderly initialization and shutdown of modules in a resilient manner.

## Decision

We opted for a modular architecture based on crates, where the project is
divided into independent sub-modules compiled separately. Each crate
encapsulates specific services and logics (e.g., core server crate, providers
crate, EventBus crate, etc.), but all operate in an integrated manner. To
coordinate the lifecycle of modules, we introduced a central component called
an AppContext composition root (ADR-050) responsible for composing,
initializing, and resolving all services across crates. Providers register via
linkme distributed slices (ADR-023) for compile-time auto-discovery, and
`init_app()` wires them into the application at startup in mcb-infrastructure.

## Implementation

### Port Trait Definition (mcb-domain)

Ports are defined as traits in `mcb-domain/src/ports/` with `Send + Sync` bounds:

```rust
// crates/mcb-domain/src/ports/providers/embedding.rs
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

Important: All port traits must:

- Be `Send + Sync` for multi-threaded async usage
- Be object-safe (no generic methods, no `Self` returns)
- Use `async_trait` for async methods

### Provider Implementation (mcb-providers)

Providers implement port traits and register via linkme distributed slices:

```rust
// crates/mcb-providers/src/embedding/ollama.rs
use mcb_domain::ports::providers::EmbeddingProvider;

pub struct OllamaEmbeddingProvider {
    api_url: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Ollama API call implementation
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Batch embedding implementation
    }

    fn dimensions(&self) -> usize { self.model_dimensions }
    fn provider_name(&self) -> &str { "ollama" }
}
```

### Default Provider (FastEmbed)

```rust
// crates/mcb-providers/src/embedding/fastembed.rs
use mcb_domain::ports::providers::EmbeddingProvider;

pub struct FastEmbedProvider { /* ... */ }

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Real embedding via fastembed
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Batch embedding via fastembed
    }

    fn dimensions(&self) -> usize { self.model_dimensions }
    fn provider_name(&self) -> &str { "fastembed" }
}
```

### DI Provider Registration (mcb-providers)

Providers register via linkme distributed slices for compile-time auto-discovery:

```rust
// crates/mcb-providers/src/embedding/fastembed.rs
register_embedding_provider!(
    fastembed_factory, config, FASTEMBED_ENTRY,
    "fastembed", "Local FastEmbed embedding provider",
    { Ok(Arc::new(FastEmbedProvider::from_config(config)?)) }
);
```

### Service Layer with Injected Dependencies (mcb-application)

Use cases receive dependencies via constructor injection:

```rust
// crates/mcb-application/src/services/context.rs
use std::sync::Arc;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};

pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self { embedding_provider, vector_store_provider }
    }

    pub async fn embed_and_store(
        &self,
        collection: &str,
        texts: &[String],
    ) -> Result<()> {
        let embeddings = self.embedding_provider.embed_batch(texts).await?;
        self.vector_store_provider.store(collection, &embeddings).await?;
        Ok(())
    }
}
```

### Two-Layer DI Strategy

The system uses a two-layer approach for DI (see [ADR-012](012-di-strategy-two-layer-approach.md)):

**Layer 1: linkme Distributed Slices** - Compile-time auto-discovery of providers:

```rust
// Providers register via distributed_slice — DI resolves by config
// (legacy details in ADR-029, superseded by ADR-050)
let resolver = EmbeddingProviderResolver::new(config);
let provider = resolver.resolve_from_config()?;
// Defaults to FastEmbedProvider when no config override
```

**Layer 2: AppContext composition root** - manual composition root composes services from resolved providers:

```rust
// mcb-infrastructure/src/di/bootstrap.rs — AppContext manual composition root (ADR-050)
let app_context = init_app(config).await?;
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mcb_providers::embedding::FastEmbedProvider;
    use mcb_providers::vector_store::EdgeVecVectorStoreProvider;

    #[tokio::test]
    async fn test_context_service_with_default_providers() {
        let embedding_provider = Arc::new(FastEmbedProvider::default());
        let vector_store_provider = Arc::new(EdgeVecVectorStoreProvider::default());

        let service = ContextService::new(embedding_provider, vector_store_provider);

        let result = service.embed_and_store("test", &["hello".to_string()]).await;
        assert!(result.is_ok());
    }
}
```

## Consequences

This change to multiple crates improved code maintainability and scalability.
Developers can evolve modules in isolation and even publish reusable crates. The
modular architecture also facilitates unit testing and integration testing
focused per module. On the other hand, it added complexity in managing versions
between internal crates and required an orchestration layer
(AppContext composition root + linkme registry) to coordinate dependencies and
initialization order. These additional structures increase robustness at the cost
of a small coordination overhead. Overall, the decision aligned with the goal of
a pluggable and extensible design, allowing inclusion or removal of
functionalities (crates) without significantly impacting the rest of the system.

## Crate Structure

```text
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services orchestration
├── mcb-providers/       # Layer 3: Provider implementations (embedding, vector stores)
├── mcb-infrastructure/  # Layer 4: DI, config, cache, crypto, health, logging <!-- markdownlint-disable-line MD013 -->
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation rules
```

**Dependency Direction** (inward only):

```text
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓
              mcb-providers
```

## Related ADRs

- [ADR-002: Async-First Architecture](002-async-first-architecture.md)
- [ADR-029: Hexagonal Architecture](archive/superseded-029-hexagonal-architecture-dill.md) (superseded by ADR-050)
- [ADR-003: Unified Provider Architecture](003-unified-provider-architecture.md)
- [ADR-004: Event Bus (Local and Distributed)](archive/superseded-004-event-bus-local-distributed.md)
- [ADR-005: Context Cache Support (Moka and Redis)](005-context-cache-support.md)
