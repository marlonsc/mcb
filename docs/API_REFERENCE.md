# API Reference

This document provides a comprehensive reference of the MCP Context Browser public API.

## Table of Contents

-   [Core Types](#core-types)
-   [Providers](#providers)
-   [Services](#services)
-   [Utilities](#utilities)

## Core Types

### Embedding

```rust
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
    pub provider: String,
}
```

Vector representation of text with metadata.

### SearchResult

```rust
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}
```

Search Result with relevance score and source location.

### CodeChunk

```rust
pub struct CodeChunk {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: Language,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

Parsed code chunk with location and language information.

## Providers

### EmbeddingProvider Trait

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
}
```

Interface for text-to-vector conversion providers.

### VectorStoreProvider Trait

```rust
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn insert_vectors(&self, collection: &str, vectors: &[Embedding], metadata: Vec<HashMap<String, serde_json::Value>>) -> Result<Vec<String>>;
    async fn search_similar(&self, collection: &str, query_vector: &[f32], limit: usize, filter: Option<&str>) -> Result<Vec<SearchResult>>;
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;
    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>>;
    async fn flush(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
}
```

Interface for vector storage and retrieval.

## Services

### ContextService

```rust
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub async fn embed_text(&self, text: &str) -> Result<Embedding>;
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    pub async fn search_similar(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}
```

Orchestrates embedding generation and vector operations.

### IndexingService

```rust
pub struct IndexingService {
    context_service: Arc<ContextService>,
}

impl IndexingService {
    pub fn new(context_service: Arc<ContextService>) -> Self;
    pub async fn index_directory(&self, path: &Path, collection: &str) -> Result<usize>;
}
```

Handles codebase indexing and chunking.

### SearchService

```rust
pub struct SearchService {
    context_service: Arc<ContextService>,
}

impl SearchService {
    pub fn new(context_service: Arc<ContextService>) -> Self;
    pub async fn search(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}
```

Provides semantic search capabilities.

## Utilities

### Metrics

```rust
pub struct SystemMetricsCollector {
    pub fn collect_cpu_metrics(&mut self) -> CpuMetrics;
    pub fn collect_memory_metrics(&mut self) -> MemoryMetrics;
}

pub struct MetricsApiServer {
    pub async fn start(&self, addr: SocketAddr) -> Result<()>;
}
```

System monitoring and metrics collection.

### Sync

```rust
pub struct CodebaseLockManager;
impl CodebaseLockManager {
    pub async fn acquire_lock(&self, codebase_path: &str) -> Result<LockMetadata>;
    pub async fn release_lock(&self, lock_id: &str) -> Result<()>;
    pub async fn cleanup_stale_locks(&self) -> Result<usize>;
}

pub struct SyncManager {
    pub async fn sync_operation<F, Fut>(&self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>>;
}
```

Cross-process synchronization utilities.

### Routing

```rust
pub struct ProviderRouter {
    pub async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;
    pub async fn get_embedding_provider(&self, context: &ProviderContext) -> Result<Arc<dyn EmbeddingProvider>>;
}

pub struct CircuitBreaker {
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>;
}
```

Intelligent provider routing with resilience.

---

*Auto-generated API reference*
