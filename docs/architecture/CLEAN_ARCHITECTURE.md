# Clean Architecture in Memory Context Browser

## Overview

Memory Context Browser follows **Clean Architecture** principles with strict layer separation across 8 Cargo workspace crates. This document explains the architecture, layer interactions, and extension patterns.

## The 6 Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 6: MCP Protocol & Transport                           │
│ (stdio, HTTP, tool handlers)                                │
│ Crate: mcb-server                                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 5: Infrastructure & Dependency Injection              │
│ (DI container, config, caching, logging, health)            │
│ Crate: mcb-infrastructure                                   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Application Services & Use Cases                   │
│ (orchestration, business logic, registry)                   │
│ Crate: mcb-application                                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Domain Entities & Ports (Traits)                   │
│ (business rules, interfaces, errors)                        │
│ Crate: mcb-domain                                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Provider Implementations                           │
│ (embedding, vector store, cache, language chunking)         │
│ Crate: mcb-providers                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Facade & Public API                                │
│ (re-exports, public interface)                              │
│ Crate: mcb                                                  │
└─────────────────────────────────────────────────────────────┘
```

## Layer Details

### Layer 1: Facade (mcb)

**Purpose**: Public API and re-exports

**Responsibilities**:

- Re-export public types from domain, application, infrastructure
- Provide single entry point for library users
- Hide internal crate structure

**Example**:

```rust
// mcb/src/lib.rs
pub use mcb_domain::{CodeChunk, Embedding, SearchResult};
pub use mcb_application::{ContextService, SearchService};
pub use mcb_infrastructure::AppContext;
```

**Dependency**: Imports from all other crates (but only re-exports public types)

### Layer 2: Domain (mcb-domain)

**Purpose**: Business rules and domain entities

**Responsibilities**:

- Define domain entities (CodeChunk, Embedding, SearchResult)
- Define port traits (EmbeddingProvider, VectorStoreProvider, etc.)
- Define domain errors with thiserror
- No external dependencies (except thiserror, serde)

**Key Types**:

```rust
// Entities
pub struct CodeChunk { /* ... */ }
pub struct Embedding { /* ... */ }
pub struct SearchResult { /* ... */ }

// Ports (traits)
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
}

pub trait VectorStoreProvider: Send + Sync {
    async fn store(&self, embedding: Embedding) -> Result<()>;
    async fn search(&self, query: &Embedding) -> Result<Vec<SearchResult>>;
}

// Errors
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    #[error("Vector store error: {0}")]
    VectorStoreError(String),
}
```

**Dependency**: None (except standard library + thiserror)

### Layer 3: Application (mcb-application)

**Purpose**: Use cases and business logic orchestration

**Responsibilities**:

- Implement services (ContextService, SearchService, IndexingService)
- Orchestrate domain entities and ports
- Define registry system (linkme distributed slices)
- Define admin ports (IndexingOperationsInterface, PerformanceMetricsInterface)

**Key Types**:

```rust
// Services
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let embedding = self.embedding_provider.embed(query).await?;
        self.vector_store.search(&embedding).await
    }
}

// Registry (linkme distributed slices)
#[linkme::distributed_slice]
pub static EMBEDDING_PROVIDERS: [EmbeddingProviderEntry] = [..];

// Admin ports
pub trait IndexingOperationsInterface: Send + Sync {
    async fn start_indexing(&self, path: &str) -> Result<()>;
    async fn get_status(&self) -> Result<IndexingStatus>;
}
```

**Dependency**: Imports from mcb-domain (ports, entities, errors)

### Layer 4: Infrastructure (mcb-infrastructure)

**Purpose**: Dependency injection, configuration, and cross-cutting concerns

**Responsibilities**:

- Build DI container (dill Catalog)
- Load configuration (Figment)
- Provide provider handles (RwLock wrappers for runtime switching)
- Implement admin services
- Logging, health checks, caching

**Key Types**:

```rust
// DI Container
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        .add_value(config)
        .add_value(embedding_provider)
        .add_value(embedding_handle)
        .add_value(embedding_admin)
        .build()
}

// Provider Handle (runtime switching)
pub struct EmbeddingProviderHandle {
    inner: RwLock<Arc<dyn EmbeddingProvider>>,
}

impl EmbeddingProviderHandle {
    pub async fn get(&self) -> Arc<dyn EmbeddingProvider> {
        self.inner.read().await.clone()
    }

    pub async fn set(&self, provider: Arc<dyn EmbeddingProvider>) {
        *self.inner.write().await = provider;
    }
}

// Admin service
pub struct EmbeddingAdminService {
    handle: Arc<EmbeddingProviderHandle>,
}

impl EmbeddingAdminInterface for EmbeddingAdminService {
    async fn switch_provider(&self, name: &str) -> Result<()> {
        let provider = EMBEDDING_PROVIDERS.iter()
            .find(|p| p.name == name)
            .ok_or(Error::ProviderNotFound)?;

        let new_provider = (provider.factory)()?;
        self.handle.set(new_provider).await;
        Ok(())
    }
}

// AppContext (composition root)
pub struct AppContext {
    embedding_handle: Arc<EmbeddingProviderHandle>,
    embedding_admin: Arc<dyn EmbeddingAdminInterface>,
    vector_store_handle: Arc<VectorStoreProviderHandle>,
    // ... other handles and services
}
```

**Dependency**: Imports from mcb-domain (ports, entities) and mcb-application (services, registry)

### Layer 5: Providers (mcb-providers)

**Purpose**: Concrete provider implementations

**Responsibilities**:

- Implement EmbeddingProvider (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
- Implement VectorStoreProvider (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
- Implement CacheProvider (Moka, Redis, Null)
- Implement LanguageChunkingProvider (Tree-sitter based)
- Auto-register via linkme distributed slices

**Key Types**:

```rust
// Embedding provider implementation
pub struct OllamaEmbeddingProvider {
    client: reqwest::Client,
    model: String,
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Implementation
    }
}

// Auto-registration
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama local embedding provider",
    factory: || {
        Ok(Arc::new(OllamaEmbeddingProvider::new()?))
    },
};
```

**Dependency**: Imports from mcb-domain (ports, entities) and mcb-application (registry)

### Layer 6: Server (mcb-server)

**Purpose**: MCP protocol implementation and transport

**Responsibilities**:

- Implement MCP tool handlers (index, search, memory, session, etc.)
- Handle stdio and HTTP transport
- Parse MCP requests and format responses
- Error handling and logging

**Key Types**:

```rust
// MCP tool handler
pub async fn handle_search(
    context: &AppContext,
    params: SearchParams,
) -> Result<SearchResponse> {
    let service = context.get_search_service()?;
    let results = service.search(&params.query).await?;
    Ok(SearchResponse { results })
}

// Transport
pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: BufWriter<Stdout>,
}

impl StdioTransport {
    pub async fn run(&mut self, context: AppContext) -> Result<()> {
        loop {
            let request = self.read_request().await?;
            let response = self.handle_request(&request, &context).await?;
            self.write_response(&response).await?;
        }
    }
}
```

**Dependency**: Imports from all layers (facade, domain, application, infrastructure)

## Dependency Direction

```
mcb-server
    ↓
mcb-infrastructure ← mcb-providers
    ↓
mcb-application ← mcb-providers
    ↓
mcb-domain
    ↓
mcb (facade)
```

**Key Rule**: Dependencies flow INWARD only. No layer imports from outer layers.

## Extension Patterns

### Pattern 1: Adding a New Embedding Provider

1. **Implement the port** (already exists in mcb-domain):

```rust
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
}
```

1. **Create provider in mcb-providers**:

```rust
pub struct MyEmbeddingProvider {
    // Implementation
}

#[async_trait]
impl EmbeddingProvider for MyEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Implementation
    }
}
```

1. **Register via linkme**:

```rust
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static MY_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "my_provider",
    description: "My custom embedding provider",
    factory: || Ok(Arc::new(MyEmbeddingProvider::new()?)),
};
```

1. **Use in infrastructure**:

```rust
// Automatically discovered and available for switching
pub async fn switch_to_my_provider(admin: &dyn EmbeddingAdminInterface) {
    admin.switch_provider("my_provider").await?;
}
```

**No changes needed** in application, server, or domain layers!

### Pattern 2: Adding a New Service

1. **Define service in mcb-application**:

```rust
pub struct MyService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl MyService {
    pub async fn do_something(&self) -> Result<()> {
        // Implementation
    }
}
```

1. **Register in DI container** (mcb-infrastructure):

```rust
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    let my_service = Arc::new(MyService::new(
        embedding_provider.clone(),
        vector_store.clone(),
    ));

    CatalogBuilder::new()
        // ... existing registrations
        .add_value(my_service)
        .build()
}
```

1. **Add MCP tool handler** (mcb-server):

```rust
pub async fn handle_my_tool(
    context: &AppContext,
    params: MyToolParams,
) -> Result<MyToolResponse> {
    let service = context.get_my_service()?;
    service.do_something().await
}
```

### Pattern 3: Adding a New Port (Interface)

1. **Define port in mcb-domain**:

```rust
pub trait MyProvider: Send + Sync {
    async fn do_something(&self) -> Result<()>;
}
```

1. **Implement in mcb-providers**:

```rust
pub struct MyProviderImpl;

#[async_trait]
impl MyProvider for MyProviderImpl {
    async fn do_something(&self) -> Result<()> {
        // Implementation
    }
}
```

1. **Register in mcb-application** (if using registry):

```rust
#[linkme::distributed_slice]
pub static MY_PROVIDERS: [MyProviderEntry] = [..];

#[linkme::distributed_slice(MY_PROVIDERS)]
static MY_IMPL: MyProviderEntry = MyProviderEntry {
    name: "my_impl",
    factory: || Ok(Arc::new(MyProviderImpl)),
};
```

1. **Use in services** (mcb-application):

```rust
pub struct MyService {
    my_provider: Arc<dyn MyProvider>,
}
```

## Benefits

### 1. Testability

Each layer can be tested independently:

- Domain: Pure business logic, no I/O
- Application: Mock providers, test orchestration
- Infrastructure: Mock DI container
- Providers: Integration tests with real services
- Server: Mock context, test handlers

### 2. Maintainability

Clear separation of concerns:

- Domain: Business rules only
- Application: Use cases and orchestration
- Infrastructure: Cross-cutting concerns
- Providers: External integrations
- Server: Protocol handling

### 3. Extensibility

Add new providers without modifying existing code:

- New embedding provider? Add to mcb-providers + register
- New service? Add to mcb-application + register in DI
- New port? Add to mcb-domain + implement in mcb-providers

### 4. Reusability

Layers can be used independently:

- Use mcb-domain for domain models
- Use mcb-application for business logic
- Use mcb-infrastructure for DI and config
- Use mcb-server for MCP protocol

## Related Documentation

- **ADR-001**: Modular Crates Architecture
- **ADR-013**: Clean Architecture Crate Separation
- **ADR-023**: Inventory to Linkme Migration
- **ADR-029**: Hexagonal Architecture with dill
- [`docs/architecture/ARCHITECTURE.md`](./ARCHITECTURE.md) – Complete architecture overview
- [`CLAUDE.md`](../../CLAUDE.md) – Development guide
