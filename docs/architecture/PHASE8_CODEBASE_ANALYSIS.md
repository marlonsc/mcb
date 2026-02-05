# PHASE 8 CODEBASE ANALYSIS: Estrutura Atual para Integração com Libraries

**Data**: 2025-02-05  
**Status**: Análise Completada (102.6K LOC, 8 crates, 131 test files)  
**Objetivo**: Entender como integrar libraries (Ollama, Milvus, etc) na arquitetura Clean Architecture

---

## 📊 ESTRUTURA GERAL DO PROJETO

### File Structure Overview

```
crates/
├── mcb/                      (379 LOC)      - Facade crate (re-exports API)
├── mcb-domain/              (8.3K LOC)     - Layer 1: Entities, Ports (traits), Errors
├── mcb-application/         (3.1K LOC)     - Layer 2: Use Cases, Services
├── mcb-infrastructure/      (9.5K LOC)     - Layer 4: DI, config, cache, crypto
├── mcb-providers/           (14.4K LOC)    - Layer 3: Implementations (embeddings, vector stores)
├── mcb-server/              (8.9K LOC)     - Layer 5: MCP protocol, handlers, transport
├── mcb-ast-utils/           (1.0K LOC)     - Utilities for AST parsing
├── mcb-language-support/    (1.0K LOC)     - Language-specific support
└── mcb-validate/            (27.5K LOC)    - Architecture validation (Phases 1–7)

TOTAL: 102.6K LOC
```

### Dependency Direction (Strict Inward Only)

```
mcb-server (MCP Protocol)
    ↓
mcb-infrastructure (DI, Config, Cache, Crypto)
    ↓
mcb-application (Services, Use Cases)
    ↓
mcb-domain (Ports/Traits, Entities)
    ↓
mcb-providers (Implementations)
```

---

## 🏗️ CURRENT BROWSER/UI CODE

### 1. MCP Tools Structure (8 Consolidated Handlers)

**File**: `mcb-server/src/handlers/consolidated/mod.rs`

```rust
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

pub struct MemoryHandler { /* ... */ }
pub struct SessionHandler { /* ... */ }
pub struct IndexHandler { /* ... */ }
pub struct ValidateHandler { /* ... */ }
pub struct VcsHandler { /* ... */ }
pub struct AgentHandler { /* ... */ }
pub struct ProjectHandler { /* ... */ }
```

#### Tool Registry (8 Tools)

| Tool | Purpose | Handler | Lines |
|------|---------|---------|-------|
| `index` | Index operations | IndexHandler | ~150 |
| `search` | Code + Memory search | SearchHandler | 136 |
| `validate` | Analysis & validation | ValidateHandler | ~120 |
| `memory` | Memory storage, timeline, inject | MemoryHandler | ~250 |
| `session` | Session lifecycle | SessionHandler | ~200 |
| `agent` | Activity logging | AgentHandler | ~80 |
| `project` | Project workflow | ProjectHandler | ~100 |
| `vcs` | VCS operations | VcsHandler | 59 |

**Total Handler Code**: ~1095 LOC (organized by action/subcommand)

### 2. Key Handler Patterns

#### Pattern 1: Dependency Injection in Handler

```rust
// mcb-server/src/handlers/consolidated/search.rs
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SearchHandler {
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self { /* ... */ }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> { /* ... */ }
}
```

**Design Decision**: Handlers are stateless wrappers around trait-based services.

#### Pattern 2: VCS Context Injection (Phase 7)

```rust
// mcb-server/src/handlers/consolidated/memory/inject.rs
pub async fn inject_context(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let vcs_context = VcsContext::capture();  // Captures git branch, commit
    match memory_service.search_memories("", Some(filter), limit).await {
        Ok(results) => {
            // Build context with VCS metadata
            serde_json::json!({
                "vcs_context": {
                    "branch": vcs_context.branch,
                    "commit": vcs_context.commit,
                }
            })
        }
    }
}
```

**File**: `mcb-domain/src/utils/vcs_context.rs` (65 LOC)

-   Uses `OnceLock` for caching (single git invocation)
-   Batches git commands: `git rev-parse --abbrev-ref HEAD HEAD`
-   Captures: `branch`, `commit`, `repo_id`

### 3. MCP Tool Arguments & Validation

```rust
// mcb-server/src/args.rs (consolidated)
pub enum SearchResource { Code, Memory }

pub struct SearchArgs {
    pub query: String,
    pub resource: SearchResource,
    pub collection: Option<String>,
    pub limit: Option<i32>,
    pub min_score: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub session_id: Option<String>,
    // ...
}

// Uses validator crate for validation
impl SearchArgs {
    pub fn validate(&self) -> Result<(), ValidationError> { /* ... */ }
}
```

---

## 🔌 GIT INTEGRATION POINTS (PHASE 7)

### VcsContext (Location: `mcb-domain/src/utils/vcs_context.rs`)

```rust
#[derive(Clone)]
pub struct VcsContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub repo_id: Option<String>,
}

impl VcsContext {
    pub fn capture() -> Self {
        // Caches results in OnceLock to avoid repeated git calls
        // Batches: git rev-parse --abbrev-ref HEAD HEAD
        //          git config --get remote.origin.url
    }
}
```

### Integration with Memory (inject_context)

```rust
// mcb-server/src/handlers/consolidated/memory/inject.rs
// VcsContext embedded in memory observation metadata:
{
    "session_id": "sess-123",
    "observation_count": 42,
    "vcs_context": {
        "branch": "feature/search",
        "commit": "abc123..."
    }
}
```

### Branch/Commit Filtering in Memory

```rust
// mcb-domain/src/entities/memory.rs
pub struct MemoryFilter {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub session_id: Option<String>,
    // ...
}

// Used in MemoryServiceImpl::search_memories_impl
fn matches_filter(obs: &Observation, filter: &MemoryFilter) -> bool {
    if let Some(ref branch) = filter.branch
        && obs.metadata.branch.as_ref() != Some(branch)
    {
        return false;
    }
    // ... similar for commit
    true
}
```

### VCS Provider (Abstraction)

```rust
// mcb-domain/src/ports/providers/vcs.rs
pub trait VcsProvider: Send + Sync {
    async fn list_repositories(&self) -> Result<Vec<Repository>>;
    async fn compare_branches(&self, repo: &str, from: &str, to: &str) 
        -> Result<BranchComparison>;
    async fn search_branch(&self, repo: &str, pattern: &str) 
        -> Result<Vec<BranchInfo>>;
    // ...
}

// Implementation: mcb-providers/src/git/
pub struct Git2Provider { /* ... */ }
impl VcsProvider for Git2Provider { /* ... */ }
```

---

## 💾 MEMORY + SEARCH INTEGRATION

### MemoryServiceImpl (Hybrid SQLite + VectorStore)

**File**: `mcb-application/src/use_cases/memory_service.rs` (382 LOC)

```rust
const RRF_K: f32 = 60.0;              // Reciprocal Rank Fusion parameter
const HYBRID_SEARCH_MULTIPLIER: usize = 3;

pub struct MemoryServiceImpl {
    project_id: String,
    repository: Arc<dyn MemoryRepository>,      // SQLite FTS
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>, // Milvus/In-Memory
}
```

#### Key Methods

1.  **store_observation**:

-   Computes content hash
-   Generates embedding
-   Inserts into vector store
-   Stores in SQLite repo

1.  **search_memories_impl** (RRF Fusion):

   ```rust
   // Fetch more candidates initially (3x multiplier)
   let candidate_limit = limit * HYBRID_SEARCH_MULTIPLIER;
   
   // Run FTS and vector search in parallel
   let (fts_result, vector_result) = tokio::join!(
       self.repository.search_fts_ranked(query, candidate_limit),
       self.vector_store.search_similar("memories", embedding, ...)
   );
   
   // RRF scoring: 1.0 / (K + rank + 1)
   for (rank, result) in fts_results.iter().enumerate() {
       let score = 1.0 / (RRF_K + rank as f32 + 1.0);
       *rrf_scores.entry(key).or_default() += score;
   }
   // Combine and normalize
   ```

### SearchServiceImpl (Context Service Delegate)

**File**: `mcb-application/src/use_cases/search_service.rs` (98 LOC)

```rust
pub struct SearchServiceImpl {
    context_service: Arc<dyn ContextServiceInterface>,
}

impl SearchServiceInterface for SearchServiceImpl {
    async fn search(&self, collection: &str, query: &str, limit: usize) 
        -> Result<Vec<SearchResult>>
    {
        self.context_service.search_similar(collection, query, limit).await
    }
}
```

### ContextServiceImpl (Embeddings + Vector Store)

**File**: `mcb-application/src/use_cases/context_service.rs` (155 LOC)

```rust
pub struct ContextServiceImpl {
    cache: Arc<dyn CacheProvider>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceInterface for ContextServiceImpl {
    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) 
        -> Result<()>
    {
        // 1. Generate embeddings batch
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;
        
        // 2. Build metadata
        let metadata: Vec<_> = chunks.iter().map(build_chunk_metadata).collect();
        
        // 3. Insert into vector store
        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;
    }

    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>
    {
        let query_embedding = self.embedding_provider.embed(query).await?;
        self.vector_store_provider
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await
    }
}
```

### Observation Metadata & Filtering

```rust
// mcb-domain/src/entities/memory.rs
pub struct ObservationMetadata {
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub file_path: Option<String>,
    pub branch: Option<String>,    // From VcsContext
    pub commit: Option<String>,    // From VcsContext
    // ...
}

pub struct MemoryFilter {
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub observation_type: Option<ObservationType>,
    // ...
}
```

---

## 🏛️ ARCHITECTURE: DOMAIN → APPLICATION → INFRASTRUCTURE

### 1. Domain Layer Ports (Trait Definitions)

**Location**: `mcb-domain/src/ports/`

#### Embedding Provider Port

```rust
// mcb-domain/src/ports/providers/embedding.rs
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}

pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
    pub tokens_used: Option<u32>,
}
```

**Implementations in mcb-providers/src/embedding/**:

-   `NullEmbeddingProvider` (Testing)
-   `OllamaEmbeddingProvider` (Local)
-   `OpenAIEmbeddingProvider` (Cloud)
-   `VoyageAIEmbeddingProvider` (Cloud)
-   `GeminiEmbeddingProvider` (Cloud)
-   `FastEmbedProvider` (Local, optional feature)

#### Vector Store Provider Port

```rust
// mcb-domain/src/ports/providers/vector_store.rs
pub trait VectorStoreProvider: VectorStoreAdmin + Send + Sync {
    async fn insert_vectors(
        &self,
        collection: &str,
        embeddings: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>>;

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&serde_json::Value>,
    ) -> Result<Vec<SearchResult>>;

    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
}
```

**Implementations in mcb-providers/src/vector_store/**:

-   `InMemoryVectorStore` (Testing)
-   `NullVectorStore` (Development)
-   `MilvusVectorStore` (Production)
-   `EdgeVecVectorStore` (Edge computing)
-   Others: Encrypted, Filesystem

#### Cache Provider Port

```rust
// mcb-domain/src/ports/providers/cache.rs
pub trait CacheProvider: Send + Sync + std::fmt::Debug {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```

**Implementations in mcb-providers/src/cache/**:

-   `MokaCache` (In-memory, high-performance)
-   `RedisCache` (Distributed)
-   `NullCache` (Development)

### 2. Application Layer Services

**Location**: `mcb-application/src/use_cases/` & `mcb-application/src/domain_services/`

#### Trait Interfaces (Domain Services)

```rust
// mcb-application/src/domain_services/search.rs
pub trait ContextServiceInterface: Send + Sync {
    async fn initialize(&self, collection: &str) -> Result<()>;
    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    async fn search_similar(&self, collection: &str, query: &str, limit: usize) 
        -> Result<Vec<SearchResult>>;
    // ...
}

pub trait SearchServiceInterface: Send + Sync {
    async fn search(&self, collection: &str, query: &str, limit: usize) 
        -> Result<Vec<SearchResult>>;
    async fn search_with_filters(...) -> Result<Vec<SearchResult>>;
}

pub trait MemoryServiceInterface: Send + Sync {
    async fn store_observation(...) -> Result<(String, bool)>;
    async fn search_memories(query: &str, filter: Option<MemoryFilter>, limit: usize)
        -> Result<Vec<MemorySearchIndex>>;
    async fn memory_search(...) -> Result<Vec<MemorySearchIndex>>;
    // ...
}
```

#### Service Implementations

```rust
// mcb-application/src/use_cases/
pub struct ContextServiceImpl { /* ... */ }
pub struct SearchServiceImpl { /* ... */ }
pub struct MemoryServiceImpl { /* ... */ }
pub struct IndexingServiceImpl { /* ... */ }
pub struct ValidationServiceImpl { /* ... */ }
```

### 3. Infrastructure Layer: Dependency Injection

**Location**: `mcb-infrastructure/src/di/modules/domain_services.rs` (165 LOC)

```rust
pub struct ServiceDependencies {
    pub project_id: String,
    pub cache: SharedCacheProvider,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    pub event_bus: Arc<dyn EventBusProvider>,
    pub memory_repository: Arc<dyn MemoryRepository>,
    pub agent_repository: Arc<dyn AgentRepository>,
}

pub struct DomainServicesFactory;

impl DomainServicesFactory {
    pub async fn create_services(deps: ServiceDependencies) 
        -> Result<DomainServicesContainer>
    {
        // 1. Create ContextServiceImpl with embedding + vector store
        let context_service: Arc<dyn ContextServiceInterface> = 
            Arc::new(ContextServiceImpl::new(
                deps.cache.into(),
                Arc::clone(&deps.embedding_provider),
                Arc::clone(&deps.vector_store_provider),
            ));

        // 2. Create SearchServiceImpl with context service
        let search_service: Arc<dyn SearchServiceInterface> =
            Arc::new(SearchServiceImpl::new(Arc::clone(&context_service)));

        // 3. Create MemoryServiceImpl with repository + embedding + vector store
        let memory_service: Arc<dyn MemoryServiceInterface> = 
            Arc::new(MemoryServiceImpl::new(
                deps.project_id.clone(),
                deps.memory_repository,
                deps.embedding_provider,
                deps.vector_store_provider,
            ));

        // 4. Assemble container
        Ok(DomainServicesContainer {
            context_service,
            search_service,
            memory_service,
            // ...
        })
    }
}
```

### 4. Server Layer: Handler Registration

**Location**: `mcb-server/src/handlers/consolidated/` (8 handlers)

```rust
// mcb-server/src/mcp_server.rs
impl MCPServer {
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get services from app context (DI container)
        let search_handler = SearchHandler::new(
            self.app_context.search_service().clone(),
            self.app_context.memory_service().clone(),
        );

        let memory_handler = MemoryHandler::new(
            self.app_context.memory_service().clone(),
            // ...
        );

        // Register with MCP router
        self.register_tool("search", Box::new(search_handler))?;
        self.register_tool("memory", Box::new(memory_handler))?;
        // ...
    }
}
```

---

## 📋 CURRENT FILE STRUCTURE (CRATES)

### mcb-server (8.9K LOC)

```
src/
├── handlers/consolidated/          (1.5K LOC)
│   ├── agent.rs                    (80 LOC)
│   ├── index.rs                    (150 LOC)
│   ├── memory/                     (250 LOC)
│   │   ├── inject.rs               (65 LOC)  ← VCS context injection
│   │   ├── execution.rs
│   │   ├── observation.rs
│   │   ├── session.rs
│   │   ├── quality_gate.rs
│   │   └── mod.rs
│   ├── search.rs                   (136 LOC)  ← Code + Memory search
│   ├── session/                    (200 LOC)
│   ├── validate.rs                 (120 LOC)
│   └── vcs/                        (150 LOC)
│       ├── index_repo.rs
│       ├── list_repos.rs
│       ├── compare_branches.rs
│       ├── search_branch.rs
│       ├── analyze_impact.rs
│       └── mod.rs
├── args.rs                         (consolidated args: Index, Search, Memory, etc)
├── tools/
│   ├── registry.rs                 (80+ LOC) ← Tool definitions & schema
│   └── router.rs
├── admin/                          (web UI handlers)
├── transport/                      (HTTP/Stdio/WebSocket)
└── lib.rs

tests/ (131 test files)
├── handlers/
├── integration/
│   ├── golden_acceptance_integration.rs     ← Golden tests
│   ├── golden_e2e_complete_integration.rs
│   ├── golden_tools_e2e_integration.rs
│   └── full_stack_integration.rs
├── admin/
└── unit/
```

### mcb-application (3.1K LOC)

```
src/
├── use_cases/
│   ├── context_service.rs          (155 LOC)  ← Embeddings + Vector Store
│   ├── search_service.rs           (98 LOC)   ← Search orchestration
│   ├── memory_service.rs           (382 LOC)  ← Hybrid search (RRF)
│   ├── indexing_service.rs
│   ├── validation_service.rs
│   ├── agent_session_service.rs
│   └── vcs_indexing.rs
├── domain_services/
│   ├── search.rs                   (Trait interfaces)
│   ├── memory.rs                   (Trait interfaces)
│   ├── chunking.rs
│   └── indexing.rs
├── ports/
│   ├── services.rs                 (200+ LOC) ← Port interfaces
│   ├── infrastructure/
│   ├── registry/
│   └── providers/
└── decorators/
    └── instrumented_embedding.rs
```

### mcb-domain (8.3K LOC)

```
src/
├── entities/
│   ├── memory.rs                   (Observation, MemoryFilter, SessionSummary)
│   ├── code_chunk.rs               (CodeChunk)
│   └── others...
├── value_objects/
│   ├── embedding.rs                (Embedding, SearchResult)
│   └── others...
├── ports/
│   ├── providers/
│   │   ├── embedding.rs            (EmbeddingProvider trait)
│   │   ├── vector_store.rs         (VectorStoreProvider trait)
│   │   ├── vcs.rs                  (VcsProvider trait)
│   │   ├── cache.rs                (CacheProvider trait)
│   │   └── others...
│   ├── infrastructure/
│   └── repositories/
├── utils/
│   ├── vcs_context.rs              (VCS capture - Phase 7)
│   └── hash.rs
├── error.rs
└── lib.rs

tests/ (unit tests for entities, value objects)
```

### mcb-infrastructure (9.5K LOC)

```
src/
├── di/
│   ├── modules/
│   │   ├── domain_services.rs      (165 LOC) ← Service factory
│   │   └── others...
│   ├── bootstrap.rs                (App context initialization)
│   ├── resolver.rs
│   └── others...
├── config/
│   ├── types/
│   │   ├── app.rs
│   │   ├── infrastructure.rs
│   │   └── others...
│   └── providers.rs                (Config-based provider selection)
├── cache/
│   ├── provider.rs
│   ├── config.rs
│   └── others...
├── crypto/
├── logging/
├── health/
└── validation/
```

### mcb-providers (14.4K LOC)

```
src/
├── embedding/                      (Multiple providers)
│   ├── null.rs                     (NullEmbeddingProvider)
│   ├── openai.rs                   (OpenAIEmbeddingProvider)
│   ├── voyageai.rs                 (VoyageAIEmbeddingProvider)
│   ├── ollama.rs                   (OllamaEmbeddingProvider)
│   ├── gemini.rs                   (GeminiEmbeddingProvider)
│   ├── fastembed.rs                (FastEmbedProvider - optional)
│   └── anthropic.rs                (AnthropicEmbeddingProvider - optional)
├── vector_store/                   (Multiple implementations)
│   ├── in_memory.rs
│   ├── milvus.rs
│   ├── edgevec.rs
│   ├── null.rs
│   └── others...
├── cache/
│   ├── moka.rs                     (In-memory cache)
│   ├── redis.rs                    (Distributed cache)
│   └── null.rs
├── git/                            (VCS provider)
│   └── provider.rs                 (Git2Provider)
├── events/                         (Event bus implementations)
│   ├── tokio.rs                    (In-process)
│   ├── nats.rs                     (Distributed)
│   └── null.rs
└── chunking/                       (Language-specific chunking)
    └── providers.rs
```

---

## 🧪 TEST PATTERNS & ARCHITECTURE

### Test Organization (131 Test Files)

#### 1. Unit Tests (Handler Level)

**File**: `mcb-server/tests/handlers/search_code_test.rs`

```rust
#[tokio::test]
async fn test_search_code_success() {
    let search_results = create_test_search_results(5);
    let search_service = MockSearchService::new().with_results(search_results);
    let memory_service = MockMemoryService::new();
    let handler = SearchHandler::new(Arc::new(search_service), Arc::new(memory_service));

    let args = SearchArgs {
        query: "test query".to_string(),
        resource: SearchResource::Code,
        collection: Some("test".to_string()),
        limit: Some(10),
        // ...
    };

    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
    assert!(!response.is_error.unwrap_or(false));
}
```

**Pattern**:

-   Create mock services
-   Wrap in Arc<dyn Trait>
-   Pass to handler constructor
-   Test handler.handle() method

#### 2. Integration Tests (Full Stack)

**File**: `mcb-server/tests/integration/golden_acceptance_integration.rs`

```rust
#[tokio::test]
async fn test_golden_queries() {
    // Initialize real app with test config
    let mut config = AppConfig::for_test();
    config.embedding_provider = "null";      // NullEmbeddingProvider
    config.vector_store = "in-memory";       // InMemoryVectorStore
    
    let app = init_app(config).await?;
    
    // Load golden fixtures
    let queries = load_golden_queries();
    let chunks = read_sample_codebase_files();
    
    // Index
    let result = app.indexing_service.index_codebase(
        Path::new("tests/fixtures/sample_codebase"),
        "default"
    ).await?;
    assert_eq!(result.status, "completed");
    
    // Search
    for query in queries.queries {
        let start = Instant::now();
        let results = app.search_service
            .search("default", &query.query, 10)
            .await?;
        
        // Verify latency and results
        assert!(start.elapsed().as_millis() < query.max_latency_ms);
        assert!(results.len() >= query.min_results);
    }
}
```

**Pattern**:

-   Real providers (Null + In-Memory)
-   Golden fixtures (queries, expected files)
-   End-to-end flow: index → search → verify
-   Latency + Result quality assertions

#### 3. Mock Services (Test Utils)

**File**: `mcb-server/tests/test_utils/mock_services.rs`

```rust
pub struct MockSearchService {
    results: Option<Vec<SearchResult>>,
}

impl MockSearchService {
    pub fn new() -> Self { /* ... */ }
    pub fn with_results(mut self, results: Vec<SearchResult>) -> Self {
        self.results = Some(results);
        self
    }
}

#[async_trait::async_trait]
impl SearchServiceInterface for MockSearchService {
    async fn search(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        Ok(self.results
            .clone()
            .unwrap_or_default()
            .into_iter()
            .take(limit)
            .collect())
    }
}
```

**Pattern**:

-   Builder pattern for configuration
-   Clone results for reusability
-   Implement trait with optional behavior

#### 4. Feature Tests

-   **Validation feature**: Architecture validation tests
-   **Embedding features**: Provider-specific tests
-   **Vector store features**: Backend-specific tests

---

## 🔧 TECHNICAL DECISIONS (ADRs)

### ADR-001: Modular Crates Architecture

**Decision**: 8 independent crates per Clean Architecture layers  
**Rationale**: Strict dependency direction, testability, independent deployment  
**Impact**: Enables swapping providers (Ollama ↔ OpenAI) without code changes

### ADR-002: Async-First Architecture

**Decision**: Tokio throughout, async_trait for dynamic dispatch  
**Rationale**: High-concurrency MCP server, parallel search (FTS + vector)  
**Impact**: All I/O is non-blocking; required for RRF fusion

### ADR-023: Linkme Provider Registration

**Decision**: Compile-time provider discovery (replaces Inventory)  
**Rationale**: Zero runtime overhead, type-safe, no reflection  
**Impact**: Providers auto-register via `#[linkme::distributed_slice]`

### ADR-029: Hexagonal Architecture with dill

**Decision**: DI container with handles (runtime-swappable)  
**Rationale**: Testability, configuration-driven provider selection  
**Impact**: Providers can be swapped via env vars (no code rebuild)

### ADR-013: RRF (Reciprocal Rank Fusion)

**Decision**: Hybrid search with k=60 parameter  
**Rationale**: Combines BM25 (FTS) + semantic (vectors) without weighting tuning  
**Impact**: MemoryServiceImpl::search_memories achieves both precision + recall

---

## 📚 KEY TRAITS & IMPLEMENTATIONS

### Summary Table

| Trait | Location | Implementations |
|-------|----------|-----------------|
| `EmbeddingProvider` | `mcb-domain/src/ports/providers/embedding.rs` | Null, OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic |
| `VectorStoreProvider` | `mcb-domain/src/ports/providers/vector_store.rs` | InMemory, Milvus, EdgeVec, Encrypted, Filesystem, Null |
| `CacheProvider` | `mcb-domain/src/ports/providers/cache.rs` | Moka, Redis, Null |
| `VcsProvider` | `mcb-domain/src/ports/providers/vcs.rs` | Git2Provider |
| `ContextServiceInterface` | `mcb-application/src/domain_services/search.rs` | ContextServiceImpl |
| `SearchServiceInterface` | `mcb-application/src/domain_services/search.rs` | SearchServiceImpl |
| `MemoryServiceInterface` | `mcb-application/src/domain_services/memory.rs` | MemoryServiceImpl |
| `IndexingServiceInterface` | `mcb-application/src/ports/services.rs` | IndexingServiceImpl |

---

## 🎯 RECOMENDAÇÕES: COMO USAR LIBRARIES

### Pattern 1: Adding a New Embedding Provider

**Step 1**: Create implementation in `mcb-providers/src/embedding/newprovider.rs`

```rust
pub struct NewEmbeddingProvider {
    client: Arc<NewClient>,
    model: String,
}

#[async_trait::async_trait]
impl EmbeddingProvider for NewEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let response = self.client.embed(text).await?;
        Ok(Embedding {
            vector: response.embedding,
            model: self.model.clone(),
            dimensions: response.dimensions,
            tokens_used: Some(response.tokens),
        })
    }
}
```

**Step 2**: Add linkme registration

```rust
#[cfg(feature = "embedding-newprovider")]
pub struct NewEmbeddingProvider { /* ... */ }

#[cfg(feature = "embedding-newprovider")]
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
fn register_new() -> (&'static str, ProviderFactory) {
    ("newprovider", Box::new(|| {
        Arc::new(NewEmbeddingProvider::new())
    }))
}
```

**Step 3**: Add feature flag in `Cargo.toml`

```toml
[features]
embedding-newprovider = []
```

**Step 4**: Use via config

```bash
export EMBEDDING_PROVIDER=newprovider
```

### Pattern 2: Adding a New Vector Store

**Step 1**: Create implementation in `mcb-providers/src/vector_store/newstore.rs`

```rust
pub struct NewVectorStore {
    client: Arc<NewClient>,
}

#[async_trait::async_trait]
impl VectorStoreProvider for NewVectorStore {
    async fn insert_vectors(
        &self,
        collection: &str,
        embeddings: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        // Insert logic
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&serde_json::Value>,
    ) -> Result<Vec<SearchResult>> {
        // Search logic
    }
}
```

**Step 2**: Register with linkme (same pattern as embeddings)

**Step 3**: Use via config

```bash
export VECTOR_STORE_PROVIDER=newstore
```

### Pattern 3: Adding Memory Filtering

**Step 1**: Extend `MemoryFilter` in `mcb-domain/src/entities/memory.rs`

```rust
pub struct MemoryFilter {
    pub session_id: Option<String>,
    pub branch: Option<String>,
    // Add new filter:
    pub tag_operator: Option<TagOperator>, // AND/OR
}
```

**Step 2**: Update `matches_filter` in `MemoryServiceImpl`

```rust
fn matches_filter(obs: &Observation, filter: &MemoryFilter) -> bool {
    // Existing checks...
    
    // Add new filter logic
    if let Some(ref operator) = filter.tag_operator {
        match operator {
            TagOperator::And => {
                // All tags must match
            }
            TagOperator::Or => {
                // Any tag must match
            }
        }
    }
    true
}
```

**Step 3**: Update `MemoryArgs` in `mcb-server/src/args.rs` to expose via MCP

### Pattern 4: Extending VCS Context Capture

**Current** (Phase 7): Captures `branch`, `commit`, `repo_id`

**Extension**: Add author, message, timestamp

```rust
// mcb-domain/src/utils/vcs_context.rs
#[derive(Clone)]
pub struct VcsContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub repo_id: Option<String>,
    pub author: Option<String>,         // NEW
    pub commit_message: Option<String>, // NEW
    pub timestamp: Option<i64>,         // NEW
}

impl VcsContext {
    pub fn capture() -> Self {
        // git log -1 --format=%an%n%s%n%ct
        // Parse output to populate new fields
    }
}
```

### Pattern 5: RRF Parameter Tuning

**Current**: k=60, multiplier=3

**To adjust for use case**:

```rust
// mcb-application/src/use_cases/memory_service.rs
const RRF_K: f32 = 60.0;  // Increase for more balanced fusion
const HYBRID_SEARCH_MULTIPLIER: usize = 3;  // Increase for more candidates

// Formula: score = 1.0 / (K + rank + 1)
// Higher K → more uniform scores (less top-weighted)
// Lower K → more top-weighted (first results dominate)
```

### Pattern 6: Integration with Libraries

**Example: Adding Milvus support**

1.  **Trait already exists**: `VectorStoreProvider` in domain
2.  **Implement in providers**:

   ```rust
   pub struct MilvusVectorStore {
       client: Arc<MilvusClient>,
   }
   impl VectorStoreProvider for MilvusVectorStore { /* ... */ }
   ```

1.  **Register with linkme**: Auto-discoverable
2.  **Configure via env**: `VECTOR_STORE_PROVIDER=milvus`
3.  **Factory in infrastructure**:

   ```rust
   match provider_name {
       "milvus" => Arc::new(MilvusVectorStore::new(config)?),
       // ...
   }
   ```

---

## 📌 INTEGRATION POINTS SUMMARY

```
Handler Layer (mcb-server)
    ↓ inject SearchHandler, MemoryHandler, etc
Application Layer (mcb-application)
    ↓ use ContextServiceInterface, SearchServiceInterface, MemoryServiceInterface
Domain Layer (mcb-domain)
    ├─ EmbeddingProvider trait
    ├─ VectorStoreProvider trait
    ├─ CacheProvider trait
    ├─ VcsProvider trait
    └─ MemoryFilter, VcsContext, Observation
    ↓
Infrastructure Layer (mcb-infrastructure)
    ├─ Config-driven provider selection
    └─ DomainServicesFactory assembles all
    ↓
Providers Layer (mcb-providers)
    ├─ EmbeddingProvider implementations
    ├─ VectorStoreProvider implementations
    ├─ CacheProvider implementations
    └─ VcsProvider implementations (Git2Provider)
```

---

## 🎓 NEXT STEPS FOR LIBRARY INTEGRATION

1.  **Choose Target**: Ollama (embeddings), Milvus (vector store), Redis (cache)?
2.  **Create Implementation**: New file in `mcb-providers/src/{target}/`
3.  **Add Trait Methods**: Implement all required trait methods
4.  **Register with Linkme**: Add `#[linkme::distributed_slice]` macro
5.  **Add Feature Flag**: In `Cargo.toml` with optional dependency
6.  **Test**: Unit tests in provider crate, integration tests with golden fixtures
7.  **Configure**: Via `PROVIDER_NAME` environment variable
8.  **Document**: Update `docs/CONFIGURATION.md` with new Option

---

**Phase 8 Complete**: Ready for library integrations! ✅
