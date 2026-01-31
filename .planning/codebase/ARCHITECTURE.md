# Architecture

**Analysis Date:** 2026-01-31

## Pattern Overview

**Overall:** Clean Architecture with Hexagonal Design (Ports & Adapters) + Handle-based Dependency Injection

**Key Characteristics:**
- Strict layering: Domain → Application → Infrastructure → Server/Providers
- Port-based abstraction: All external dependencies are traits defined in domain
- Handle-based DI: Runtime-swappable providers via RwLock wrappers
- Linkme registry: Auto-registration of provider implementations at compile time
- Feature-gated provider selection: Minimal builds with selective compilation

## Layers

**Domain Layer:**
- Purpose: Core business logic, entities, value objects, and port definitions
- Location: `crates/mcb-domain/src/`
- Contains: `entities/` (CodeChunk), `value_objects/` (Embedding, SearchResult), `ports/` (trait definitions), `error.rs`, `events/`, `repositories/`
- Depends on: Nothing (no external dependencies, pure business rules)
- Used by: Application layer (uses entities and ports)

**Application Layer:**
- Purpose: Use cases and business logic orchestration
- Location: `crates/mcb-application/src/`
- Contains: `use_cases/` (IndexingServiceImpl, SearchServiceImpl, ContextServiceImpl), `domain_services/` (service interfaces), `ports/` (application-level ports), `decorators/` (cross-cutting concerns)
- Depends on: mcb-domain (entities, ports, errors)
- Used by: Infrastructure and server layers

**Provider Layer:**
- Purpose: Concrete implementations of domain port interfaces
- Location: `crates/mcb-providers/src/`
- Contains: `embedding/` (OpenAI, Ollama, VoyageAI, Gemini, FastEmbed), `vector_store/` (Milvus, EdgeVec, Filesystem, Encrypted, InMemory), `cache/` (Moka, Redis), `language/` (AST-based code chunking), `events/` (EventBus implementations), `hybrid_search/`
- Depends on: mcb-domain (port traits)
- Used by: Infrastructure layer (via linkme registry)

**Infrastructure Layer:**
- Purpose: Cross-cutting concerns: DI, configuration, caching, crypto, logging
- Location: `crates/mcb-infrastructure/src/`
- Contains: `di/` (Handle-based DI, AppContext, provider resolvers), `config/` (Figment-based config loading), `cache/` (caching abstractions), `crypto/` (AES-GCM encryption), `health/` (health checks), `logging/` (tracing integration), `validation/` (optional architecture validation)
- Depends on: mcb-domain, mcb-application, mcb-providers (linkme registry access)
- Used by: Server layer

**Server/MCP Layer:**
- Purpose: MCP protocol implementation, HTTP/stdio transport, tool handlers
- Location: `crates/mcb-server/src/`
- Contains: `mcp_server.rs` (MCP ServerHandler impl), `handlers/` (tool handlers), `transport/` (HTTP/stdio), `init.rs` (bootstrap), `tools/` (registry and routing), `admin/` (web UI for provider switching)
- Depends on: mcb-application, mcb-infrastructure
- Used by: Binary entrypoint

## Data Flow

**Indexing Flow:**

1. User calls `index_codebase` tool via MCP
2. `IndexCodebaseHandler.handle()` validates path, calls `IndexingServiceImpl.index_codebase()`
3. `IndexingServiceImpl` discovers files, skips `[".git", "node_modules", "target", "__pycache__"]`, filters by extensions `["rs", "py", "js", "ts", "java", "cpp", "c", "go"]`
4. For each file, calls `LanguageChunkingProvider` (AST-based) to split into semantic chunks
5. For each chunk, calls `EmbeddingProvider` to generate vector embeddings
6. Calls `VectorStoreProvider.store()` to persist chunks and embeddings
7. Publishes `IndexingCompleted` event via `EventBusProvider`
8. Returns `IndexingResult` with statistics

**Search Flow:**

1. User calls `search_code` tool with natural language query
2. `SearchCodeHandler.handle()` calls `SearchServiceImpl.search()`
3. `SearchServiceImpl` delegates to `ContextServiceImpl.search_similar()`
4. `ContextServiceImpl` calls `EmbeddingProvider` to embed query
5. Calls `VectorStoreProvider.search()` with embedded query vector
6. Returns top-k `SearchResult` objects with similarity scores and file context

**State Management:**

- **Indexing State**: `IndexingOperationsInterface` tracks async indexing progress (operation_id, status, file_counts)
- **Provider State**: Handles (e.g., `EmbeddingProviderHandle`) wrap providers in `RwLock` for runtime switching
- **Cache State**: `CacheProvider` caches embeddings and search results by collection
- **Transient State**: Events published to `EventBusProvider` (Tokio channels or NATS)

## Key Abstractions

**Service Interfaces (Application Layer):**
- `IndexingServiceInterface` (`crates/mcb-application/src/domain_services/search.rs`) - File discovery, chunking, indexing
- `SearchServiceInterface` - Query execution with result filtering
- `ContextServiceInterface` - Central orchestrator for embedding + vector store operations
- `ValidationServiceInterface` - Architecture and code validation (optional)

**Port Interfaces (Domain Layer):**
- `EmbeddingProvider` (`crates/mcb-domain/src/ports/providers/embedding.rs`) - Generates vectors from text
- `VectorStoreProvider` - Stores and searches embeddings
- `CacheProvider` - Caches embeddings and search results
- `LanguageChunkingProvider` - Splits code into semantic chunks using AST
- `EventBusProvider` - Publishes domain events
- `IndexingOperationsInterface` - Tracks async indexing state
- `ValidationServiceInterface` - Validates architecture rules

**Tool Handlers (Server Layer):**
- `IndexCodebaseHandler` - Orchestrates indexing operation
- `SearchCodeHandler` - Executes semantic search
- `ClearIndexHandler` - Clears vector store collection
- `GetIndexingStatusHandler` - Returns indexing progress
- `ValidateArchitectureHandler` - Validates codebase structure
- `ValidateFileHandler` - Validates single file
- `AnalyzeComplexityHandler` - Returns code complexity metrics
- `ListValidatorsHandler` - Lists available validation rules
- `GetValidationRulesHandler` - Returns validation rules by category

**Provider Handles (Infrastructure Layer):**
- `EmbeddingProviderHandle` (RwLock wrapper) - Runtime-swappable embedding provider
- `VectorStoreProviderHandle` - Runtime-swappable vector store
- `CacheProviderHandle` - Runtime-swappable cache
- `LanguageChunkingProviderHandle` - Runtime-swappable language processor

## Entry Points

**Binary Entry Point:**
- Location: `crates/mcb/src/main.rs`
- Triggers: `cargo run` or `./mcb` binary
- Responsibilities: Parse CLI args, select operating mode (server/standalone/client), call `init::run()`

**Server Bootstrap:**
- Location: `crates/mcb-server/src/init.rs` - `run()` function
- Triggers: HTTP daemon startup or stdio server
- Responsibilities:
  1. Load configuration (Figment-based, supports TOML + env vars)
  2. Initialize logging (tracing)
  3. Resolve providers via linkme registry
  4. Wrap providers in handles for runtime switching
  5. Create `AppContext` (DI container)
  6. Create domain services via `DomainServicesFactory`
  7. Create `McpServer` with service dependencies
  8. Start transport (HTTP or stdio)

**Operating Modes:**
- `server_mode` - HTTP daemon listening on port (default 8080), stdio transport optional
- `standalone` - Local providers, stdio transport (CLI-friendly)
- `client` - Connects to remote server via HTTP

**MCP Tool Invocation:**
- Location: `crates/mcb-server/src/mcp_server.rs` - `McpServer::call_tool()`
- Triggers: MCP client calls tool
- Responsibilities:
  1. Route tool name to handler via `route_tool_call()`
  2. Deserialize tool arguments
  3. Call handler method (e.g., `IndexCodebaseHandler::handle()`)
  4. Return formatted MCP result

## Error Handling

**Strategy:** Custom error types with context, no panics in production code

**Patterns:**
- Domain errors: `mcb_domain::error::Error` (thiserror-based)
- MCP errors: Convert to `rmcp::ErrorData` with structured messages
- Infrastructure errors: Wrapped in `ErrorContext` for additional context
- File I/O: Logged but not panicked, included in `IndexingResult.errors` collection
- Parser errors: AST parsing failures recorded per-file, indexing continues

**Error Conversion:**
- `domain::Result<T>` → `MCP ErrorData` (handlers convert via `ResponseFormatter`)
- Configuration errors: Fail fast on startup (no fallbacks for missing config)
- Provider initialization: Fail if selected provider unavailable

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` crate with structured logging
- Configured via `logging` section in config
- Levels: trace, debug, info, warn, error
- Entry points: `init.rs` calls `mcb_infrastructure::logging::init_logging()`

**Validation:**
- Input validation: `validator` crate (serde-powered) on handler arguments
- Path validation: Handler checks paths exist and are directories
- Architecture validation: Optional feature (`validation`), via `mcb-validate` crate

**Authentication:**
- No built-in auth for stdio transport
- HTTP transport: Basic support via `Authorization` header (custom implementation in `auth.rs`)
- Admin API: Requires auth for provider switching

**Performance Monitoring:**
- Per-operation timing: Handlers use `Instant` to measure execution time
- Caching: `CacheProvider` reduces redundant embeddings (TTL configurable)
- Event publishing: Async (non-blocking) via `EventBusProvider`

## Clean Architecture Principles Applied

1. **Dependency Inversion**: All dependencies flow inward (Server → App → Domain), never outward
2. **Port-based Abstraction**: No direct imports of concrete implementations in domain/app layers
3. **Linkme Registry**: Providers auto-register at compile time without explicit DI configuration
4. **Handle Pattern**: Wraps providers in RwLock for runtime switching without recompilation
5. **Feature-gated Builds**: Each provider can be enabled/disabled via Cargo features
6. **No Leaky Abstractions**: MCP protocol details stay in server layer, domain stays pure business logic

---

*Architecture analysis: 2026-01-31*
