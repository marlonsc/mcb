# ADR 001: Provider Pattern Architecture

## Status

**Implemented** (v0.1.1)

> Fully implemented with Clean Architecture + Shaku DI across 6 crates:
>
> **Port Traits** (defined in `crates/mcb-domain/src/ports/`):
> All traits extend `shaku::Interface` for DI compatibility.
>
> -   **Provider Ports** (`crates/mcb-domain/src/ports/providers/`):
>     -   `EmbeddingProvider` - Text-to-vector conversion (6 implementations)
>     -   `VectorStoreProvider` - Vector storage/retrieval (6 implementations)
>     -   `CacheProvider` - Caching abstraction
>     -   `CryptoProvider` - Encryption/hashing
>     -   `LanguageChunkingProvider` - AST-based code chunking (12 languages)
>
> -   **Infrastructure Ports** (`crates/mcb-domain/src/ports/infrastructure/`):
>     -   `SyncProvider` - Low-level sync operations
>     -   `SnapshotProvider` - Codebase snapshot management
>     -   `EventPublisher` - Domain event publishing
>
> -   **Admin Ports** (`crates/mcb-domain/src/ports/admin.rs`):
>     -   `PerformanceMetrics` - Performance monitoring
>     -   `IndexingOperations` - Indexing management
>
> **Provider Implementations** (in `crates/mcb-providers/src/`):
>
> -   Embedding: OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null
> -   Vector Store: Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null
> -   Cache: Moka, Redis, Null
> -   Language: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin

## Context

The MCP Context Browser needs to support multiple AI providers (OpenAI, Ollama, VoyageAI) and vector databases (Milvus, In-Memory, Filesystem) without creating tight coupling between the core business logic and external service implementations. The system must be extensible to add new providers without modifying existing code, and support testing with mock implementations.

Following Clean Architecture principles, we need:

1.  Port traits (interfaces) defined in the domain layer, independent of implementations
2.  Provider implementations (adapters) in a separate providers crate
3.  Dependency injection for wiring implementations to ports at runtime
4.  Null implementations for testing without external services

## Decision

Implement a provider pattern using Rust traits as port interfaces, with Shaku for dependency injection and a two-layer DI strategy (see [ADR-012](012-di-strategy-two-layer-approach.md)).

Key architectural elements:

1.  **Port traits** extending `shaku::Interface` in `mcb-domain`
2.  **Provider implementations** with `#[derive(Component)]` in `mcb-providers`
3.  **Shaku modules** for compile-time DI composition in `mcb-infrastructure`
4.  **Runtime factories** for configuration-driven provider selection

## Implementation

### Port Trait Definition (mcb-domain)

Ports are defined as traits extending `shaku::Interface` for DI compatibility:

```rust
// crates/mcb-domain/src/ports/providers/embedding.rs
use shaku::Interface;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Interface + Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

Important: All port traits must:

-   Extend `shaku::Interface` (which implies `'static + Send + Sync` with thread_safe feature)
-   Be object-safe (no generic methods, no `Self` returns)
-   Use `async_trait` for async methods

### Provider Implementation (mcb-providers)

Providers implement ports and are registered as Shaku components:

```rust
// crates/mcb-providers/src/embedding/openai.rs
use shaku::Component;
use mcb_domain::ports::providers::EmbeddingProvider;

#[derive(Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct OpenAIEmbeddingProvider {
    #[shaku(default)]
    api_key: String,
    #[shaku(default)]
    model: String,
    #[shaku(default)]
    client: reqwest::Client,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // OpenAI API call implementation
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Batch embedding implementation
    }

    fn dimensions(&self) -> usize { 1536 }
    fn provider_name(&self) -> &str { "openai" }
}
```

### Null Provider for Testing

```rust
// crates/mcb-providers/src/embedding/null.rs
use shaku::Component;
use mcb_domain::ports::providers::EmbeddingProvider;

#[derive(Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct NullEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for NullEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Embedding> {
        Ok(Embedding::zeros(128))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        Ok(texts.iter().map(|_| Embedding::zeros(128)).collect())
    }

    fn dimensions(&self) -> usize { 128 }
    fn provider_name(&self) -> &str { "null" }
}
```

### DI Module Registration (mcb-infrastructure)

Shaku modules register components for DI:

```rust
// crates/mcb-infrastructure/src/di/modules/embedding_module.rs
use shaku::module;
use mcb_providers::embedding::NullEmbeddingProvider;

module! {
    pub EmbeddingModuleImpl {
        components = [NullEmbeddingProvider],
        providers = []
    }
}
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

    pub async fn embed_and_store(&self, collection: &str, texts: &[String]) -> Result<()> {
        let embeddings = self.embedding_provider.embed_batch(texts).await?;
        self.vector_store_provider.store(collection, &embeddings).await?;
        Ok(())
    }
}
```

### Two-Layer DI Strategy

The system uses a two-layer approach for DI (see [ADR-012](012-di-strategy-two-layer-approach.md)):

**Layer 1: Shaku Modules** - Provide null implementations as defaults for testing:

```rust
// Testing with Shaku modules (null providers)
let container = DiContainerBuilder::new().build().await?;
// Uses NullEmbeddingProvider, NullVectorStoreProvider, etc.
```

**Layer 2: Runtime Factories** - Create production providers from configuration:

```rust
// Production with factories
let embedding = EmbeddingProviderFactory::create(&config.embedding, None)?;
let vector_store = VectorStoreProviderFactory::create(&config.vector_store, crypto)?;
let services = DomainServicesFactory::create_services(
    cache, crypto, config, embedding, vector_store, chunker
).await?;
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mcb_providers::embedding::NullEmbeddingProvider;
    use mcb_providers::vector_store::NullVectorStoreProvider;

    #[tokio::test]
    async fn test_context_service_with_null_providers() {
        let embedding_provider = Arc::new(NullEmbeddingProvider);
        let vector_store_provider = Arc::new(NullVectorStoreProvider);

        let service = ContextService::new(embedding_provider, vector_store_provider);

        let result = service.embed_and_store("test", &["hello".to_string()]).await;
        assert!(result.is_ok());
    }
}
```

## Consequences

### Positive Consequences

-   **Clean Architecture Compliance**: Strict separation between ports (domain) and adapters (providers)
-   **High Extensibility**: New providers added in `mcb-providers` without touching domain/application
-   **Testability**: Null providers enable unit testing without external services
-   **Type Safety**: Shaku verifies DI wiring at compile time
-   **Runtime Flexibility**: Factories enable configuration-driven provider selection

### Negative Consequences

-   **Learning Curve**: Developers must understand Shaku macros and Clean Architecture
-   **Boilerplate**: Each provider needs trait implementation + Component derive
-   **Two Patterns**: Testing uses Shaku modules, production uses factories

## Crate Structure

```
crates/
├── mcb-domain/src/ports/           # Port trait definitions
│   ├── providers/                   # Provider ports (embedding, vector_store, etc.)
│   ├── infrastructure/              # Infrastructure ports (sync, snapshot, etc.)
│   └── admin.rs                     # Admin ports
├── mcb-providers/src/               # Provider implementations
│   ├── embedding/                   # 6 embedding providers
│   ├── vector_store/               # 6 vector store providers
│   ├── cache/                       # Cache providers
│   └── language/                    # 12 language processors
├── mcb-application/src/services/   # Use cases with injected ports
└── mcb-infrastructure/src/di/      # DI modules and factories
    ├── modules/                     # Shaku module definitions
    └── factory/                     # Runtime provider factories
```

## Related ADRs

-   [ADR-004: Multi-Provider Strategy](004-multi-provider-strategy.md)
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md)
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md)

## References

-   [Shaku Documentation](https://docs.rs/shaku)
-   [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
-   [Port and Adapter Pattern](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software))
