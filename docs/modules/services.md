# Services Module

**Source**: `src/application/`
**Traits**: 4 service interfaces in `src/domain/ports/services.rs`

Orchestrates the semantic code search workflow - from codebase ingestion to search results.

## Overview

The services module contains core business logic that powers the semantic code search platform. Each service encapsulates specific capabilities that work together to deliver code intelligence.

All services implement interface traits defined in `src/domain/ports/services.rs` for DI compatibility.

## Service Interface Traits

All service interfaces extend `shaku::Interface`:

```rust
pub trait ContextServiceInterface: Interface + Send + Sync {
    fn initialize(&self) -> impl Future<Output = Result<()>> + Send;
    fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> impl Future<Output = Result<()>> + Send;
    fn search_similar(&self, collection: &str, query: &str, limit: usize) -> impl Future<Output = Result<Vec<SearchResult>>> + Send;
    fn embed_text(&self, text: &str) -> impl Future<Output = Result<Embedding>> + Send;
    fn clear_collection(&self, collection: &str) -> impl Future<Output = Result<()>> + Send;
    fn embedding_dimensions(&self) -> usize;
}

pub trait SearchServiceInterface: Interface + Send + Sync {
    fn search(&self, collection: &str, query: &str, limit: usize) -> impl Future<Output = Result<Vec<SearchResult>>> + Send;
}

pub trait IndexingServiceInterface: Interface + Send + Sync {
    fn index_codebase(&self, path: &Path, collection: &str) -> impl Future<Output = Result<IndexingResult>> + Send;
    fn get_status(&self) -> IndexingStatus;
    fn clear_collection(&self, collection: &str) -> impl Future<Output = Result<()>> + Send;
}

pub trait ChunkingOrchestratorInterface: Interface + Send + Sync {
    fn process_files(&self, files: &[PathBuf], collection: &str) -> impl Future<Output = Result<Vec<CodeChunk>>> + Send;
    fn process_file(&self, path: &Path, collection: &str) -> impl Future<Output = Result<Vec<CodeChunk>>> + Send;
}
```

## Services

### ContextService

Coordinates embedding generation and vector storage operations.

**Constructor** (simplified in v0.1.0+):
```rust
pub fn new_with_providers(
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
) -> Self
```

Note: The `hybrid_search_provider` parameter was removed as it was unused.

**Responsibilities**:

1.   Generate embeddings via AI providers
2.   Store and retrieve vectors
3.   Handle batch processing
4.   Collect performance metrics

**Related**: [providers/embedding](./providers.md), [core/types](./core.md)

### IndexingService

Processes codebases and creates searchable vector indexes.

**Responsibilities**:

1.   Repository scanning and file discovery
2.   Language detection and AST parsing
3.   Incremental indexing with change detection
4.   Chunk generation and metadata extraction

**Related**: [chunking module](../../src/domain/chunking/), [core/types](./core.md)

### SearchService

Executes semantic similarity searches across indexed codebases.

**Responsibilities**:

1.   Query processing and embedding generation
2.   Vector similarity search execution
3.   Result ranking and filtering
4.   Response caching and optimization

**Related**: [providers/vector_store](./providers.md), [core/hybrid_search](./core.md)

### ChunkingOrchestrator

Coordinates batch chunking operations across files.

**Responsibilities**:

1.   Process multiple files in parallel
2.   Coordinate with CodeChunker implementation
3.   Handle file batching and error recovery

## Integration Points

### AI Providers

1.   OpenAI, Ollama, Gemini, VoyageAI, FastEmbed
2.   Intelligent routing with failover
3.   See [providers module](./providers.md)

### Vector Storage

1.   Milvus (production), InMemory (development), EdgeVec, Filesystem
2.   See [providers module](./providers.md)

### MCP Protocol

1.   Standardized interface with AI assistants
2.   See [server module](./server.md)

## Key Exports

```rust
pub use context::ContextService;
pub use indexing::IndexingService;
pub use search::SearchService;
```

## File Structure

```text
src/application/
├── context.rs              # Embedding and vector operations
├── indexing/
│   ├── service.rs          # Codebase ingestion and processing
│   ├── chunking_orchestrator.rs  # Batch chunking coordination
│   └── file_discovery.rs   # File discovery utilities
├── search.rs               # Query processing and ranking
└── mod.rs                  # Module coordination

src/domain/ports/services.rs  # Service interface traits
```

## Testing

See [tests/services/](../../tests/services/) for service-specific tests.

## Cross-References

-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-  **Core Types**: [core.md](./core.md)
-  **Providers**: [providers.md](./providers.md)
-  **Server**: [server.md](./server.md)
-  **Domain Ports**: [domain.md](./domain.md)

---

*Updated 2026-01-13 - Added service interface traits and API changes*
