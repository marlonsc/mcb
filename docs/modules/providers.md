# Providers Module

**Source**: `src/adapters/providers/`

Extensible provider system for AI embedding and vector storage services.

## Overview

The providers module implements a trait-based abstraction layer for AI and storage services. This enables flexible deployment with multiple providers, intelligent routing, and automatic failover.

## Provider Categories

### Embedding Providers (`embedding/`)

Transform text into vector embeddings.

| Provider | Model | Dimensions | Use Case |
|----------|-------|------------|----------|
| OpenAI | text-embedding-3-small | 1536 | Production |
| Ollama | nomic-embed-text | 768 | Self-hosted |
| Gemini | text-embedding-004 | 768 | Alternative |
| VoyageAI | voyage-3-lite | 512 | Specialized |
| FastEmbed | local models | varies | Privacy-first |
| Mock | fixed vectors | 128 | Testing |

**Trait**:

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

### Vector Store Providers (`vector_store/`)

Store and search vector embeddings.

| Provider | Storage | Use Case |
|----------|---------|----------|
| Milvus | Distributed DB | Production scale |
| InMemory | RAM | Development/testing |
| Filesystem | Local files | Simple persistence |
| Encrypted | Encrypted files | Sensitive data |
| EdgeVec | Edge-optimized | Low-latency edge |

**Trait**:

```rust
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()>;
    async fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete_collection(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}
```

### Routing System (`routing/`)

Intelligent provider selection and management.

\1-  **ProviderRouter**- Main routing orchestration
\1-  **CircuitBreaker**- Failure detection and recovery
\1-  **HealthMonitor**- Continuous health checking
\1-  **CostTracker**- Usage and cost monitoring
\1-  **FailoverManager**- Automatic provider switching

## File Structure

```text
src/adapters/providers/
├── embedding/
│   ├── fastembed.rs    # Local embeddings
│   ├── gemini.rs       # Google Gemini
│   ├── mod.rs          # Trait definition
│   ├── null.rs         # Mock provider
│   ├── ollama.rs       # Self-hosted
│   ├── openai.rs       # OpenAI API
│   └── voyageai.rs     # VoyageAI
├── routing/
│   ├── circuit_breaker.rs
│   ├── cost_tracker.rs
│   ├── failover.rs
│   ├── health.rs
│   ├── metrics.rs
│   ├── mod.rs
│   └── router.rs
├── vector_store/
│   ├── edgevec.rs
│   ├── encrypted.rs
│   ├── filesystem.rs
│   ├── in_memory.rs
│   ├── milvus.rs
│   ├── mod.rs
│   └── null.rs
└── mod.rs
```

## Key Exports

```rust
// Traits
pub trait EmbeddingProvider;
pub trait VectorStoreProvider;

// Implementations
pub use embedding::{OpenAIEmbeddingProvider, OllamaEmbeddingProvider};
pub use vector_store::{MilvusVectorStoreProvider, InMemoryVectorStore};

// Routing
pub use routing::{ProviderRouter, CircuitBreaker, HealthMonitor};
```

## Testing

34 provider tests plus 25+ routing tests. See [tests/](../../tests/).

## Cross-References

\1-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
\1-  **Core Types**: [core.md](./core.md) (Embedding, SearchResult)
\1-  **Services**: [services.md](./services.md) (uses providers)
\1-  **Configuration**: [Claude.md](../../CLAUDE.md) (environment setup)
