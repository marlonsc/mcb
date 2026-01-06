# Architecture Overview

## 🏗️ System Architecture

MCP Context Browser implements a clean, modular architecture following SOLID principles, designed for maintainability and extensibility.

### Architecture Layers

The system is organized into clear architectural layers with well-defined responsibilities:

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Context Browser                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 MCP Server Layer                   │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │   MCP       │ │   Tool     │ │   Message   │   │    │
│  │  │ Protocol    │ │ Handlers   │ │ Processing  │   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Business Logic Layer                  │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │ Context     │ │ Indexing    │ │ Search      │   │    │
│  │  │ Service     │ │ Service     │ │ Service     │   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                Provider Layer                       │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │ Embedding   │ │ Vector      │ │ File        │   │    │
│  │  │ Providers   │ │ Store       │ │ System      │   │    │
│  │  │             │ │ Providers   │ │ Provider    │   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 Core Layer                          │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │   Types     │ │   Errors    │ │   Traits    │   │    │
│  │  │ & Models    │ │   Handling  │ │ & Interfaces│   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

#### Layer Responsibilities

- **MCP Server Layer**: Protocol implementation, message handling, tool dispatch
- **Business Logic Layer**: Core application logic, service orchestration
- **Provider Layer**: Pluggable implementations for external dependencies
- **Core Layer**: Fundamental types, traits, and error handling

### Current Implementation Status

| Component | Status | Description |
|-----------|--------|-------------|
| `core` | ✅ **Complete** | Types, errors, and traits fully implemented |
| `providers/embedding` | ⚠️ **Basic** | Mock provider implemented, framework ready for real providers |
| `providers/vector_store` | ⚠️ **Basic** | In-memory provider implemented, ready for database integration |
| `services` | ⚠️ **Partial** | Basic structure exists, core logic needs implementation |
| `server` | ⚠️ **Partial** | MCP protocol framework exists, tools need completion |
| `config` | 🚧 **Stub** | Basic structs exist, loading mechanism needed |

### Data Flow

```
Client Request → MCP Server → Tool Handler → Business Service → Provider → Response
```

#### Request Processing Flow

1. **MCP Server** receives JSON-RPC message via stdio
2. **Message Processing** parses and validates the request
3. **Tool Handler** dispatches to appropriate business logic
4. **Business Service** orchestrates operations using providers
5. **Provider** executes external operations (embeddings, storage)
6. **Response** flows back through the same layers

### Provider Architecture

The system uses a provider pattern for extensibility, allowing different implementations of the same interface.

#### Current Providers

- **EmbeddingProvider**: Converts text to vector embeddings
  - `MockEmbeddingProvider`: Fixed-dimension vectors for testing
  - Framework ready for: OpenAI, Ollama, VoyageAI implementations

- **VectorStoreProvider**: Stores and retrieves vector embeddings
  - `InMemoryVectorStoreProvider`: RAM-based storage for development
  - Framework ready for: Milvus, Pinecone, Qdrant implementations

#### Provider Registration

```rust
// Example provider registration
let mut registry = ProviderRegistry::new();
registry.register_embedding_provider("mock", Arc::new(MockEmbeddingProvider::new()));
registry.register_vector_store_provider("memory", Arc::new(InMemoryVectorStoreProvider::new()));
```

### Error Handling

The system implements comprehensive error handling with custom error types:

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    // ... additional variants
}
```

### Dependency Injection

The system uses constructor injection for testability and maintainability:

```rust
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            embedding_provider,
            vector_store_provider,
        }
    }
}
```

### Testing Strategy

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **MCP Protocol Tests**: Validate protocol compliance
- **Provider Tests**: Test different provider implementations

### Performance Considerations

- **Async Processing**: Tokio runtime for non-blocking operations
- **Memory Management**: Arc/RwLock for thread-safe shared state
- **Streaming**: Prepared for large codebase processing
- **Caching**: Framework ready for caching implementations

### Security Design

- **Input Validation**: All inputs validated before processing
- **Error Handling**: Sensitive information not exposed in errors
- **Resource Limits**: Configurable limits on processing resources
- **Access Control**: Framework prepared for authentication/authorization

This architecture provides a solid foundation for building a robust, extensible MCP server while maintaining clean separation of concerns and testability.
