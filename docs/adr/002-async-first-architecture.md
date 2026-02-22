<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
<!-- markdownlint-disable MD025 -->
adr: 2
title: Async-First Architecture
status: IMPLEMENTED
created:
updated: 2026-02-05
related: [1, 3, 12, 13]
supersedes: []
superseded_by: []
implementation_status: Complete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

## ADR 002: Async-First Architecture

## Status

Accepted

> Fully implemented with Tokio async runtime across 7 crates in the Clean
> Architecture workspace.
>
> **Async Distribution by Crate**:
>
> - `mcb-domain` - Port traits with async methods (`async_trait`)
> - `mcb-application` - Use case services (ContextService, SearchService,
>   IndexingService)
> - `mcb-providers` - Provider implementations (embedding, vector_store, cache)
> - `mcb-infrastructure` - DI bootstrap (AppContext + linkme), factories, event bus
> - `mcb-server` - MCP protocol handlers, admin API
>
> All provider ports use `async_trait` with `Send + Sync` bounds.
> DI via AppContext composition root + linkme distributed slices (ADR-050; ADR-029 superseded).
> Structured concurrency with `tokio::spawn` and async channels.

## Context

The Memory Context Browser handles AI operations (embedding generation, vector
searches) and large codebase processing that require high performance and
concurrency. The system needs to handle multiple concurrent users, process large
codebases efficiently, and integrate with external APIs that may have high
latency.

Key performance requirements:

- Handle 1000+ concurrent users
- Process codebases with 1000+ files efficiently
- Maintain sub-500ms response times for queries
- Support streaming and background processing
- Integrate with external APIs (OpenAI, vector databases)

Traditional synchronous programming would create bottlenecks and poor resource
utilization for these I/O-bound operations.

## Decision

Adopt an async-first architecture using Tokio as the async runtime throughout
the entire system. All provider interfaces use async traits, and the application
is designed for high concurrency from the ground up.

Key architectural decisions:

- Tokio as the primary async runtime
- Async traits for all provider interfaces
- Structured concurrency with Tokio::spawn
- Async channels for inter-task communication
- Hyper for HTTP client operations
- Futures and streams for data processing pipelines

### Consequences

Async-first architecture provides excellent performance and concurrency but
requires careful error handling and increases code complexity.

### Positive Consequences

- **High Performance**: Efficient handling of concurrent operations and I/O
- **Scalability**: Support for thousands of concurrent users
- **Resource Efficiency**: Better CPU and memory utilization
- **Future-Proof**: Aligns with modern async programming patterns
- **Integration**: Natural fit with async HTTP clients and databases

### Negative Consequences

- **Complexity**: Async code is harder to reason about and debug
- **Error Handling**: Async error propagation is more complex
- **Testing**: Async tests require special handling
- **Learning Curve**: Steeper learning curve for team members
- **Debugging**: Stack traces are less informative in async contexts

## Alternatives Considered

### Alternative 1: Synchronous Architecture

- **Description**: Traditional blocking I/O with thread pools for concurrency
- **Pros**: Simpler code, easier debugging, familiar patterns
- **Cons**: Poor performance for I/O operations, limited concurrency
- **Rejection Reason**: Cannot meet performance requirements for AI operations
  and concurrent users

### Alternative 2: Mixed Sync/Async

- **Description**: Sync core with async wrappers for external operations
- **Pros**: Gradual adoption, less complexity
- **Cons**: Inconsistent patterns, performance bottlenecks at boundaries
- **Rejection Reason**: Creates architectural inconsistency and performance
  issues

### Alternative 3: Actor Model (Actix)

- **Description**: Use Actix for actor-based concurrency instead of Tokio
- **Pros**: High-level abstractions, built-in supervision
- **Cons**: Additional complexity, less ecosystem support
- **Rejection Reason**: Tokio has better ecosystem support and performance for
  our use case

## Implementation Notes

### Async Runtime Configuration (mcb-server)

```rust
// crates/mcb-server/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configure Tokio runtime for optimal performance
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("mcp-worker")
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack
        .enable_io()
        .enable_time()
        .build()?;

    runtime.block_on(run_server(None))
}
```

### Async Port Traits (mcb-domain)

Port traits use `Send + Sync` bounds for async DI compatibility:

```rust
// crates/mcb-domain/src/ports/providers/embedding.rs
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Default: delegate to embed_batch
        let embeddings = self.embed_batch(&[text.to_owned()]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| Error::embedding("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

Important: Port traits in `mcb-domain` must:

- Be `Send + Sync` for multi-threaded async usage
- Use `async_trait` for async methods
- Be object-safe for `Arc<dyn Trait>` usage

### Async Provider Implementations (mcb-providers)

Providers implement async port traits and register via linkme distributed slices:

```rust
// crates/mcb-providers/src/embedding/ollama.rs
use async_trait::async_trait;
use mcb_domain::ports::providers::EmbeddingProvider;

pub struct OllamaEmbeddingProvider { /* HTTP client, model config */ }

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Ollama API doesn't support batch - process sequentially
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let response_data = self.fetch_single_embedding(text).await?;
            results.push(self.parse_embedding(&response_data)?);
        }
        Ok(results)
    }

    fn dimensions(&self) -> usize { /* model-dependent */ 768 }
    fn provider_name(&self) -> &str { "ollama" }
}

// Auto-registration via linkme distributed slice
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama local embedding provider",
    factory: ollama_factory,
};
```

### Structured Concurrency (mcb-application)

Use case services orchestrate async operations with structured concurrency:

```rust
// crates/mcb-application/src/services/indexing.rs
pub struct IndexingService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
    language_chunker: Arc<dyn LanguageChunkingProvider>,
}

impl IndexingService {
    pub async fn process_codebase(&self, path: &Path) -> Result<IndexingStats> {
        // Create a task scope for structured concurrency
        let (stats_tx, stats_rx) = tokio::sync::mpsc::channel(100);

        // Spawn background tasks
        let file_processing = tokio::spawn(async move {
            self.process_files_concurrently(path, stats_tx).await
        });

        let metadata_update = tokio::spawn(async move {
            self.update_metadata_concurrently(path).await
        });

        // Wait for all tasks to complete
        let (file_result, metadata_result) = tokio::try_join!(
            file_processing,
            metadata_update
        )?;

        file_result?;
        metadata_result?;

        // Collect final statistics
        let mut total_stats = IndexingStats::default();
        while let Some(stats) = stats_rx.recv().await {
            total_stats.merge(stats);
        }

        Ok(total_stats)
    }
}
```

### Error Handling in Async Code (mcb-server)

MCP handlers use timeout and cancellation patterns:

```rust
// crates/mcb-server/src/handlers/index.rs
pub async fn handle_index_request(
    &self,
    request: IndexRequest
) -> Result<IndexResponse> {
    // Use timeout for external operations
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        self.indexing_service.index (action=start)(&request.path)
    ).await
    .map_err(|_| Error::timeout("Indexing timed out"))??;

    Ok(IndexResponse { stats: result })
}

// crates/mcb-server/src/handlers/search.rs
pub async fn handle_search_request(
    &self,
    request: SearchRequest
) -> Result<SearchResponse> {
    // Handle cancellation gracefully
    let mut search_task = self.search_service.search(request.query);

    tokio::select! {
        result = search_task => {
            Ok(SearchResponse { results: result? })
        }
        _ = tokio::signal::ctrl_c() => {
            Err(Error::cancelled("Search was cancelled"))
        }
    }
}
```

### Testing Async Code

Tests use default providers for isolation:

```rust
// crates/mcb-application/tests/services_test.rs
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use mcb_providers::embedding::FastEmbedProvider;
    use mcb_providers::vector_store::EdgeVecVectorStoreProvider;

    #[tokio::test]
    async fn test_embedding_with_default_provider() {
        let provider = Arc::new(FastEmbedProvider::default());

        // Test async operation with real provider
        let embedding = provider.embed("test text").await.unwrap();
        assert!(!embedding.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_embedding_batch() {
        let provider = Arc::new(FastEmbedProvider::default());

        // Test concurrent embedding
        let texts = vec!["text1".to_string(), "text2".to_string(), "text3".to_string()];
        let embeddings = provider.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
    }
}

// Integration test with AppContext composition root (ADR-050)
// crates/mcb-infrastructure/tests/di_test.rs
#[tokio::test]
async fn test_full_async_flow_with_di() {
    use mcb_infrastructure::di::bootstrap::init_app;

    let app_context = init_app(config).await.unwrap();
    let embedding: Arc<dyn EmbeddingProvider> = app_context.embedding_handle().get();

    let result = embedding.embed("test").await;
    assert!(result.is_ok());
}
```

## Update for v0.3.0: Hybrid Parallelization with Rayon

**Date**: 2026-01-14

As MCB evolves to include CPU-intensive code analysis features (v0.3.0+), the
async-first design has been extended to support hybrid parallelization:

### Updated Strategy

- **Tokio**: I/O-bound operations (file reads, network calls, database queries,
  vector search)
- **Rayon**: CPU-bound operations (AST parsing, complexity calculation, graph
  analysis)
- **Pattern**: Wrap Rayon in `tokio::task::spawn_blocking` to bridge sync CPU
  work with async I/O

#### Rationale

1. **Tokio for I/O**: Tokio's event-driven architecture is optimal for I/O-bound
   work
2. **Rayon for Compute**: Rayon's work-stealing scheduler is proven for
   CPU-bound parallelism
3. **PMAT Integration**: Upcoming PMAT analysis code uses Rayon extensively
   with proven performance
4. **No Conflicts**: Tokio and Rayon are complementary and don't interfere with
   each other

### Implementation Pattern

```rust
#[async_trait]
pub trait CodeAnalyzer: Send + Sync {
    async fn validate (
        action=analyze
    )(&self, path: &Path) -> Result<ComplexityReport> {
        // 1. Read file (I/O - Tokio)
        let content = tokio::fs::read_to_string(path).await?;

        // 2. Compute complexity (CPU - Rayon, wrapped in spawn_blocking)
        let report = tokio::task::spawn_blocking(move || {
            // Rayon parallelism for AST analysis
            self.compute_complexity(&content)
        }).await??;

        Ok(report)
    }
}

fn compute_complexity(content: &str) -> Result<ComplexityReport> {
    // Rayon for parallel AST node processing
    let nodes = parse_ast(content)?;

    let metrics: Vec<_> = nodes
        .par_iter()  // Rayon's parallel iterator
        .map(|node| calculate_node_complexity(node))
        .collect();

    Ok(ComplexityReport { metrics })
}
```

### Benefits

- ✅ Tokio remains the primary runtime for all async coordination
- ✅ Rayon's work-stealing keeps CPU cores busy during analysis
- ✅ No context switching between runtimes
- ✅ Straightforward to test and reason about
- ✅ Maintains clean async/sync boundaries

### Performance Implications

- **I/O Operations**: Unchanged (Tokio handles efficiently)
- **CPU Operations**: Improved parallelism (Rayon fully utilizes CPU cores)
- **Context Switching**: Minimal (spawn_blocking reuses Tokio's worker threads)
- **Memory**: Slight increase for Rayon work-stealing queues (negligible)

## Related ADRs

- [ADR-001: Modular Crates Architecture]
(001-modular-crates-architecture.md) -
  Provider interfaces with async traits
- [ADR-003: Unified Provider Architecture & Routing]
(003-unified-provider-architecture.md) -
  Async provider selection and failover
- [ADR-012: Two-Layer DI Strategy]
(012-di-strategy-two-layer-approach.md) -
  Async initialization in factories
- [ADR-013: Clean Architecture Crate Separation]
(013-clean-architecture-crate-separation.md) -
  Crate organization

## References

- [Tokio Documentation](https://tokio.rs/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Structured Concurrency]
(<https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/>) <!-- markdownlint-disable-line MD013 -->
- [Rayon: Data Parallelism](https://docs.rs/rayon/latest/rayon/)
- [Tokio spawn_blocking]
(<https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html>) <!-- markdownlint-disable-line MD013 -->
- [linkme Documentation](https://docs.rs/linkme) (compile-time discovery in current DI; see ADR-050)
