# Codebase Structure

**Analysis Date:** 2026-01-31

## Directory Layout

```
mcb/
├── crates/                          # Workspace crates
│   ├── mcb/                         # Facade crate (main binary + lib)
│   │   ├── src/
│   │   │   ├── main.rs             # Binary entrypoint
│   │   │   ├── cli/                # CLI subcommands (serve, validate)
│   │   │   └── lib.rs              # Public API re-exports
│   │   └── Cargo.toml
│   │
│   ├── mcb-domain/                 # Domain layer (business logic only)
│   │   ├── src/
│   │   │   ├── mod.rs              # Module root
│   │   │   ├── entities/           # Core business entities (CodeChunk)
│   │   │   ├── value_objects/      # Immutable values (Embedding, SearchResult)
│   │   │   ├── ports/              # Trait definitions (all external contracts)
│   │   │   │   ├── providers/      # Provider port interfaces
│   │   │   │   │   ├── embedding.rs          # EmbeddingProvider trait
│   │   │   │   │   ├── vector_store.rs       # VectorStoreProvider trait
│   │   │   │   │   ├── cache.rs              # CacheProvider trait
│   │   │   │   │   ├── language_chunking.rs  # LanguageChunkingProvider trait
│   │   │   │   │   ├── validation.rs         # ValidationProvider trait
│   │   │   │   │   └── metrics_analysis.rs   # MetricsProvider trait
│   │   │   │   ├── infrastructure/  # Infrastructure port interfaces
│   │   │   │   │   ├── events.rs    # EventBusProvider trait
│   │   │   │   │   └── [others]
│   │   │   │   ├── admin.rs         # AdminInterface (IndexingOperationsInterface)
│   │   │   │   └── services.rs      # ValidationServiceInterface
│   │   │   ├── events/             # Domain events
│   │   │   ├── error.rs            # Custom error types (thiserror)
│   │   │   ├── repositories/       # Repository traits
│   │   │   └── constants.rs        # Domain constants
│   │   └── Cargo.toml
│   │
│   ├── mcb-application/            # Application layer (use cases)
│   │   ├── src/
│   │   │   ├── lib.rs              # Module root
│   │   │   ├── use_cases/          # Use case implementations
│   │   │   │   ├── indexing_service.rs      # IndexingServiceImpl
│   │   │   │   ├── search_service.rs        # SearchServiceImpl
│   │   │   │   ├── context_service.rs       # ContextServiceImpl
│   │   │   │   ├── validation_service.rs    # ValidationServiceImpl
│   │   │   │   └── mod.rs
│   │   │   ├── domain_services/   # Service interfaces
│   │   │   │   ├── chunking.rs     # ChunkingServiceInterface
│   │   │   │   ├── indexing.rs     # IndexingServiceInterface
│   │   │   │   ├── search.rs       # SearchServiceInterface, ContextServiceInterface
│   │   │   │   └── mod.rs
│   │   │   ├── ports/             # Application-level port definitions
│   │   │   │   ├── registry/      # Provider registry (linkme slices)
│   │   │   │   ├── providers/     # Re-exports of domain provider ports
│   │   │   │   ├── infrastructure/ # Application-specific infrastructure ports
│   │   │   │   └── services.rs    # Service interface definitions
│   │   │   ├── decorators/        # SOLID decorators (cross-cutting concerns)
│   │   │   └── mod.rs
│   │   └── Cargo.toml
│   │
│   ├── mcb-providers/              # Provider implementations
│   │   ├── src/
│   │   │   ├── lib.rs              # Module root
│   │   │   ├── embedding/          # EmbeddingProvider implementations
│   │   │   │   ├── openai.rs       # OpenAI embedding provider
│   │   │   │   ├── ollama.rs       # Ollama local embedding
│   │   │   │   ├── voyageai.rs     # VoyageAI embedding
│   │   │   │   ├── gemini.rs       # Google Gemini embedding
│   │   │   │   ├── fastembed.rs    # FastEmbed local embedding
│   │   │   │   └── null.rs         # Null provider (testing)
│   │   │   ├── vector_store/       # VectorStoreProvider implementations
│   │   │   │   ├── milvus.rs       # Milvus vector database
│   │   │   │   ├── edgevec.rs      # EdgeVec in-memory store
│   │   │   │   ├── filesystem/     # Filesystem-based storage
│   │   │   │   ├── encrypted.rs    # AES-GCM encrypted store
│   │   │   │   └── null.rs         # Null provider
│   │   │   ├── cache/              # CacheProvider implementations
│   │   │   │   ├── moka.rs         # Moka local cache
│   │   │   │   ├── redis.rs        # Redis distributed cache
│   │   │   │   └── null.rs         # Null provider
│   │   │   ├── language/           # LanguageChunkingProvider implementations
│   │   │   │   ├── rust.rs         # Rust AST chunker
│   │   │   │   ├── python.rs       # Python AST chunker
│   │   │   │   ├── javascript.rs   # JavaScript/TypeScript chunker
│   │   │   │   ├── [others]/       # Java, C, Go, etc.
│   │   │   │   └── mod.rs
│   │   │   ├── events/             # EventBusProvider implementations
│   │   │   │   ├── tokio.rs        # Tokio channel-based event bus
│   │   │   │   ├── nats.rs         # NATS event bus
│   │   │   │   └── null.rs         # Null provider
│   │   │   ├── hybrid_search/      # Hybrid search (BM25 + semantic)
│   │   │   ├── chunking/           # Code chunking utilities
│   │   │   ├── http/               # HTTP client abstractions
│   │   │   ├── utils/              # Shared provider utilities
│   │   │   └── constants.rs        # Provider-specific constants
│   │   └── Cargo.toml
│   │
│   ├── mcb-infrastructure/         # Infrastructure layer
│   │   ├── src/
│   │   │   ├── mod.rs              # Module root
│   │   │   ├── di/                 # Dependency injection
│   │   │   │   ├── bootstrap.rs    # AppContext creation
│   │   │   │   ├── catalog.rs      # dill IoC container setup
│   │   │   │   ├── modules/        # DI modules
│   │   │   │   │   ├── domain_services.rs  # Domain services factory
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── resolver.rs     # Provider resolution via linkme registry
│   │   │   │   ├── provider_resolvers.rs # Specific provider resolvers
│   │   │   │   ├── handles.rs      # RwLock-wrapped provider handles
│   │   │   │   └── mod.rs
│   │   │   ├── config/             # Configuration management
│   │   │   │   ├── types/          # Config struct definitions
│   │   │   │   └── mod.rs          # Figment-based config loading
│   │   │   ├── cache/              # Cache abstractions
│   │   │   │   ├── provider.rs     # SharedCacheProvider trait
│   │   │   │   └── mod.rs
│   │   │   ├── crypto/             # Encryption/cryptography
│   │   │   │   └── service.rs      # AES-GCM encryption
│   │   │   ├── health.rs           # Health check endpoints
│   │   │   ├── logging.rs          # Tracing initialization
│   │   │   ├── validation/         # Architecture validation (feature-gated)
│   │   │   ├── routing/            # Provider routing
│   │   │   ├── constants.rs        # Infrastructure constants
│   │   │   └── error_ext.rs        # Error context extensions
│   │   └── Cargo.toml
│   │
│   ├── mcb-server/                 # Server/MCP layer
│   │   ├── src/
│   │   │   ├── lib.rs              # Module root
│   │   │   ├── main.rs             # DEPRECATED (see crates/mcb/src/main.rs)
│   │   │   ├── mcp_server.rs       # MCP ServerHandler implementation
│   │   │   ├── init.rs             # Bootstrap and operating mode selection
│   │   │   ├── builder.rs          # McpServerBuilder
│   │   │   ├── args.rs             # Tool argument definitions (serde + validator)
│   │   │   ├── formatter.rs        # Response formatting for tools
│   │   │   ├── handlers/           # Tool handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── index_codebase.rs
│   │   │   │   ├── search_code.rs
│   │   │   │   ├── get_indexing_status.rs
│   │   │   │   ├── clear_index.rs
│   │   │   │   ├── validate_architecture.rs
│   │   │   │   ├── validate_file.rs
│   │   │   │   ├── list_validators.rs
│   │   │   │   ├── get_validation_rules.rs
│   │   │   │   └── analyze_complexity.rs
│   │   │   ├── tools/              # Tool registry and routing
│   │   │   │   ├── registry.rs     # Tool definitions and schemas
│   │   │   │   ├── router.rs       # Route tool calls to handlers
│   │   │   │   └── mod.rs
│   │   │   ├── transport/          # Protocol transports
│   │   │   │   ├── http.rs         # HTTP transport (Rocket)
│   │   │   │   ├── stdio.rs        # Stdio transport
│   │   │   │   ├── config.rs       # Transport config
│   │   │   │   └── mod.rs
│   │   │   ├── admin/              # Admin API
│   │   │   │   ├── sse.rs          # Server-sent events
│   │   │   │   ├── web/            # Web UI for provider switching
│   │   │   │   └── mod.rs
│   │   │   ├── session/            # Session management
│   │   │   ├── auth.rs             # Authentication helpers
│   │   │   ├── constants.rs        # Server constants
│   │   │   ├── collection_mapping.rs # Collection name normalization
│   │   │   └── mod.rs
│   │   └── Cargo.toml
│   │
│   └── mcb-validate/               # Validation tooling (feature: "validation")
│       ├── src/
│       │   ├── lib.rs
│       │   └── dependency.rs       # Dependency analysis
│       └── Cargo.toml
│
├── docs/                            # Documentation
│   ├── adr/                         # Architecture Decision Records (31 ADRs)
│   ├── architecture/                # Architecture documentation
│   └── migration/                   # Migration guides
│
├── config/                          # Configuration templates
│   └── mcb.toml                     # Configuration template
│
├── .planning/                       # This folder - Codebase analysis docs
│   ├── codebase/
│   │   ├── ARCHITECTURE.md         # (This file) - Layer patterns, data flows
│   │   ├── STRUCTURE.md            # (This file) - Directory layout, file purposes
│   │   ├── CONVENTIONS.md          # (Future) - Code style, patterns
│   │   └── TESTING.md              # (Future) - Test organization, TDD patterns
│
├── make/                            # Makefile modules
│   ├── Makefile.release.mk         # Release targets
│   └── [other makefiles]
│
├── tests/                           # Integration tests (if any)
│
├── Cargo.toml                       # Workspace root
├── Cargo.lock                       # Dependency lock
├── Makefile                         # Main makefile
├── docker-compose.yml              # Local development services
└── .env.example                     # Environment variable template
```

## Directory Purposes

**`crates/mcb/`** - Facade crate
- Re-exports public API from all layers
- Contains CLI binary entrypoint
- Ensures mcb-providers is in dependencies (for linkme registration)

**`crates/mcb-domain/`** - Domain layer
- Pure business logic, no external dependencies
- Trait definitions only - no implementations
- Entity and value object definitions
- Custom error types

**`crates/mcb-application/`** - Application layer
- Use case implementations (IndexingServiceImpl, SearchServiceImpl, etc.)
- Orchestrates domain entities and providers
- Service interfaces that bind domain to infrastructure
- Depends on domain only

**`crates/mcb-providers/`** - Provider implementations
- Concrete implementations of domain port traits
- Auto-registers via linkme distributed slices
- Feature-gated compilation (one provider per feature)
- No business logic - just adapter implementations

**`crates/mcb-infrastructure/`** - Infrastructure layer
- DI container (dill-based with AppContext)
- Configuration loading (Figment-based)
- Caching abstractions
- Crypto services
- Health checks and logging
- Bridges providers to application layer

**`crates/mcb-server/`** - Server/MCP layer
- MCP protocol implementation (ServerHandler trait)
- HTTP and stdio transports
- Tool handlers (MCP request handlers)
- Bootstrap and initialization logic
- Admin web UI for runtime provider switching

## Key File Locations

**Entry Points:**
- `crates/mcb/src/main.rs` - Binary entrypoint (calls init::run)
- `crates/mcb-server/src/init.rs` - Server initialization and mode selection
- `crates/mcb-server/src/mcp_server.rs` - MCP protocol handler

**Configuration:**
- `crates/mcb-infrastructure/src/config/` - Figment config loading
- `config/mcb.toml` - Configuration template (XDG defaults)
- `crates/mcb/src/args.rs` - CLI argument parsing (clap)

**Core Business Logic:**
- `crates/mcb-domain/src/entities/` - CodeChunk, Embedding
- `crates/mcb-domain/src/value_objects/` - SearchResult, Embedding values
- `crates/mcb-application/src/use_cases/indexing_service.rs` - File discovery and chunking
- `crates/mcb-application/src/use_cases/search_service.rs` - Search orchestration
- `crates/mcb-application/src/use_cases/context_service.rs` - Embedding + vector store coordination

**Port Definitions:**
- `crates/mcb-domain/src/ports/providers/` - EmbeddingProvider, VectorStoreProvider, CacheProvider
- `crates/mcb-domain/src/ports/services.rs` - ValidationServiceInterface

**Tool Handlers:**
- `crates/mcb-server/src/handlers/index_codebase.rs` - Indexing MCP tool
- `crates/mcb-server/src/handlers/search_code.rs` - Search MCP tool
- `crates/mcb-server/src/handlers/validate_architecture.rs` - Validation tool
- All handlers follow same pattern: validate args → call service → format response

**DI & Wiring:**
- `crates/mcb-infrastructure/src/di/bootstrap.rs` - AppContext creation
- `crates/mcb-infrastructure/src/di/modules/domain_services.rs` - Service factory
- `crates/mcb-infrastructure/src/di/resolver.rs` - Provider resolution from linkme registry

**Transports:**
- `crates/mcb-server/src/transport/http.rs` - HTTP server (Rocket framework)
- `crates/mcb-server/src/transport/stdio.rs` - Stdio MCP transport

## Naming Conventions

**Files:**
- Service implementations: `{name}_service.rs` (e.g., `indexing_service.rs`)
- Handler implementations: `{tool_name}_handler.rs` (e.g., `index_codebase.rs`)
- Port/trait definitions: `{domain}.rs` or `{domain}/` (e.g., `providers/embedding.rs`)
- Tests: `{subject}_test.rs` or `tests/` subdirectories by type
- Configuration: Uppercase for modules (`Config`, `AppConfig`), lowercase for files

**Directories:**
- Plural for collections of similar items: `entities/`, `providers/`, `handlers/`, `tests/`
- Singular for single responsibility: `config/`, `error.rs`, `logging.rs`
- Subsystem grouping: `di/`, `admin/`, `transport/`

**Types:**
- Traits: `{Noun}Interface` or `{Verb}Provider` (e.g., `EmbeddingProvider`, `IndexingServiceInterface`)
- Implementations: `{Noun}Impl` (e.g., `IndexingServiceImpl`, `OllamaEmbeddingProvider`)
- Handlers: `{Noun}Handler` (e.g., `IndexCodebaseHandler`)
- Errors: `{Context}Error` (e.g., `ValidationError`)
- Value objects: PascalCase (e.g., `SearchResult`, `Embedding`)

**Functions:**
- Service methods: `action_resource()` (e.g., `index_codebase()`, `search_similar()`)
- Validators: `validate_X()` (e.g., `validate_request()`)
- Getters: `{attribute}()` (e.g., `indexing_service()`)
- Factory methods: `new()` or `create_X()` (e.g., `new()`, `create_services()`)

## Where to Add New Code

**New Feature (e.g., "Add hybrid search toggle"):**
- Primary code: `crates/mcb-application/src/use_cases/search_service.rs` (orchestration)
- Port updates: `crates/mcb-domain/src/ports/providers/hybrid_search.rs` (if new capability)
- Handler: `crates/mcb-server/src/handlers/search_code.rs` (update tool args)
- Tests: `crates/mcb-application/tests/unit/` (service logic), `crates/mcb-server/tests/handlers/` (tool integration)

**New Embedding Provider (e.g., "Add Anthropic embeddings"):**
- Implementation: `crates/mcb-providers/src/embedding/anthropic.rs` (new file)
- Register: Add linkme `#[distributed_slice]` in file
- Feature gate: Add `embedding-anthropic` feature in `Cargo.toml`
- Config: `crates/mcb-infrastructure/src/config/types/` (add provider type)
- Resolver: `crates/mcb-infrastructure/src/di/resolver.rs` (add resolution logic)

**New MCP Tool (e.g., "Add export_index tool"):**
- Handler: `crates/mcb-server/src/handlers/export_index.rs` (new file)
- Arguments: Add to `crates/mcb-server/src/args.rs` via struct
- Registry: Add to `crates/mcb-server/src/tools/registry.rs` via `ToolDefinitions`
- Routing: Add to `crates/mcb-server/src/tools/router.rs` match statement
- Handler mod: Export in `crates/mcb-server/src/handlers/mod.rs`

**New Utility/Helper:**
- Shared across crates: `crates/mcb-infrastructure/src/utils/` (infrastructure concern) or new domain entity
- Crate-specific: `{crate}/src/utils/` or appropriate module
- Tests: Co-located with source in `tests/unit/` subdirectory

## Special Directories

**`crates/mcb-providers/src/language/`** - Generated? No. Committed? Yes.
- Contains AST parsers for 14 languages
- Uses tree-sitter grammar files (static, checked in)
- Language detection logic determines which parser to use

**`crates/mcb-infrastructure/src/di/`** - Generated? No. Committed? Yes.
- Handle-based DI and AppContext
- Provider resolution via linkme registry (compile-time, not runtime)
- No generated code, pure Rust

**`docs/adr/`** - Architecture Decision Records
- 31 ADRs documenting major design choices
- Helpful for understanding why patterns exist
- Read before making architectural changes

**`.planning/codebase/`** - This folder
- Analysis documents for `/gsd` command system
- ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md
- Read by `/gsd:plan-phase` and `/gsd:execute-phase` tools

---

*Structure analysis: 2026-01-31*
