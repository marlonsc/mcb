<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 13
title: Clean Architecture Crate Separation
status: IMPLEMENTED
created:
updated: 2026-02-05
related: [1, 2, 3, 6, 7, 11, 12, 27, 31]
supersedes: []
superseded_by: []
implementation_status: Complete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 013: Clean Architecture Crate Separation

## Status

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.

Implemented (v0.1.1) - Six crates
Updated (v0.1.2) - Added mcb-validate as 7th crate

## Context

As Memory Context Browser evolved from a monolithic architecture to a production-ready system, the codebase grew to include multiple providers, complex DI patterns, validation systems, and protocol handlers. A monolithic structure created several challenges:

1. **Coupling**: Changes to infrastructure affected domain logic
2. **Testability**: Testing required loading entire application context
3. **Compilation**: Small changes triggered full rebuilds
4. **Clarity**: No clear boundaries for where code belongs
5. **Dependency Direction**: Violations of dependency inversion were easy to introduce

The Clean Architecture pattern, as described by Robert C. Martin, addresses these concerns through strict layer separation with dependencies pointing inward toward the domain.

## Decision

We organize the codebase into **seven Cargo workspace crates** following Clean Architecture principles:

### Layer 1: Domain (`mcb-domain`)

**Purpose**: Core business entities, port traits (interfaces), and domain validation rules.

Characteristics:

- Zero external dependencies (except `async_trait`, `thiserror`)
- Defines port traits with `Send + Sync` bounds for async DI compatibility
- Contains domain entities: `CodeChunk`, `Repository`, `Embedding`, `SearchResult`
- Contains value objects: `Language`, `ChunkType`, `SearchQuery`
- Defines domain errors with `thiserror`
- No implementations of external services

Key Directories:

```text
mcb-domain/src/
├── entities/           # Domain entities (CodeChunk, Codebase)
├── events/             # Domain events (DomainEvent, EventPublisher)
├── repositories/       # Repository port traits (ChunkRepository, SearchRepository)
├── value_objects/      # Value objects (Embedding, Config, Search, Types)
├── constants.rs        # Domain constants
└── error.rs            # Domain error types
```

**Dependencies**: None (except trait utilities)

### Layer 2: Application (`mcb-application`)

**Purpose**: Business logic orchestration and use case implementations.

Characteristics:

- Depends only on `mcb-domain`
- Contains use cases: `ContextService`, `SearchService`, `IndexingService`
- Orchestrates domain operations without knowing implementations
- Defines application-level ports (service interfaces)
- Contains the `ChunkingOrchestrator` for batch processing

Key Directories:

```text
mcb-application/src/
├── services/
│   ├── context.rs      # ContextService - embedding + storage coordination
│   ├── search.rs       # SearchService - semantic search
│   └── indexing.rs     # IndexingService - codebase indexing
├── domain_services/
│   └── chunking.rs     # ChunkingOrchestrator
├── ports/              # Application-level port interfaces
└── use_cases/          # Use case modules
```

**Dependencies**: `mcb-domain`

### Layer 3: Providers (`mcb-providers`)

**Purpose**: Implementations of domain port traits for external services.

Characteristics:

- Depends on `mcb-domain` (implements port traits)
- Feature-flagged providers for optional dependencies
- Contains real implementations: OpenAI, Ollama, etc. (7 embedding, 4 vector store, 12 language)
- Organized by provider category

Key Directories:

```text
mcb-providers/src/
├── embedding/          # 7 embedding providers
│   ├── fastembed.rs    # Default (local)
│   ├── ollama.rs
│   ├── openai.rs
│   ├── voyageai.rs
│   ├── gemini.rs
│   └── anthropic.rs
├── vector_store/       # 4 vector store providers
│   ├── edgevec.rs      # Default (local)
│   ├── qdrant.rs
│   ├── milvus.rs
│   └── pinecone.rs
├── cache/              # Cache providers
│   ├── moka.rs         # Default (in-memory)
│   └── redis.rs
├── events/             # Event bus providers
│   ├── tokio.rs        # Default (in-process)
│   └── nats.rs
├── language/           # 12 AST-based language processors
│   ├── rust.rs
│   ├── python.rs
│   └── ...
├── routing/            # Circuit breaker, failover, health
└── hybrid_search/      # BM25 + semantic search
```

**Dependencies**: `mcb-domain`, external SDKs (feature-gated)

### Layer 4: Infrastructure (`mcb-infrastructure`)

**Purpose**: Shared technical services and cross-cutting concerns.

Characteristics:

- Depends on `mcb-domain`, `mcb-application`, `mcb-providers`
- Contains the linkme + Handle DI system with AppContext composition root (ADR-050; ADR-029 superseded)
- Contains configuration management (Figment)
- Contains cross-cutting services (metrics, events)
- Provides factories for production provider creation

Key Directories:

```text
mcb-infrastructure/src/
├── di/
│   ├── bootstrap.rs    # Application init (init_app)
│   ├── bootstrap.rs    # AppContext manual composition root
│   ├── admin.rs        # Admin service wiring
│   └── resolvers/      # Provider resolvers (from linkme registry)
├── config/             # Configuration types (Figment)
├── infrastructure/     # Admin types (metrics, indexing ops)
├── crypto/             # Encryption services
├── health/             # Health check infrastructure
└── logging/            # Logging configuration (tracing)
```

**Dependencies**: `mcb-domain`, `mcb-application`, `mcb-providers`

### Layer 5: Server (`mcb-server`)

**Purpose**: MCP protocol implementation and HTTP API.

Characteristics:

- Depends on all other crates
- Entry point for the application
- MCP protocol handler with stdio transport
- Tool handlers (index, search, clear, status)
- Admin API endpoints

Key Directories:

```text
mcb-server/src/
├── handlers/           # MCP tool handlers
├── transport/          # Stdio transport
├── tools/              # Tool registry
├── admin/              # Admin API
├── init.rs             # Server initialization
└── main.rs             # Entry point
```

**Dependencies**: All crates

### Layer 6: Validate (`mcb-validate`)

**Purpose**: Architecture enforcement and code quality validation.

Characteristics:

- Standalone validation tool
- 30+ validators for architecture rules
- Violation trait system for unified reporting
- TOML-based configuration
- Used in CI/CD pipelines

Key Components:

- `CleanArchitectureValidator`: Layer dependency rules
- `QualityValidator`: Code quality metrics
- `OrganizationValidator`: File organization rules

**Dependencies**: Development tool, not production dependency

### Layer 7: Facade (`mcb`)

**Purpose**: Public API re-exports for external consumers.

Characteristics:

- Re-exports public types from all crates
- Provides unified interface for library users
- Minimal code, mostly re-exports

**Dependencies**: All crates

## Dependency Graph

```text
                    ┌─────────────┐
                    │   mcb-server│
                    │   (Layer 5) │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│mcb-infrastructure│ │ mcb-validate│ │       mcb       │
│    (Layer 4)     │ │  (Layer 6)  │ │   (Layer 7)     │
└────────┬────────┘ └─────────────┘ └─────────────────┘
         │
    ┌────┴────┬────────────┐
    │         │            │
    ▼         ▼            ▼
┌────────┐ ┌────────────┐ ┌─────────────┐
│mcb-app │ │mcb-providers│ │             │
│(Layer 2)│ │  (Layer 3) │ │             │
└────┬───┘ └──────┬─────┘ │             │
     │            │       │             │
     └────────────┼───────┘             │
                  │                     │
                  ▼                     │
           ┌─────────────┐              │
           │  mcb-domain │◄─────────────┘
           │   (Layer 1) │
           └─────────────┘

Arrow direction: depends on
```

## Clean Architecture Rules Enforced

1. **Dependency Rule**: Dependencies only point inward (toward domain)
2. **Abstraction Rule**: Inner layers define interfaces (ports), outer layers implement (adapters)
3. **Entity Rule**: Domain entities have no external dependencies
4. **Use Case Rule**: Application layer orchestrates, doesn't implement infrastructure

## Validation

The `mcb-validate` crate enforces these architectural rules:

```bash

# Run architecture validation
cargo run -p mcb-validate

# Key validators

# - CleanArchitectureValidator: Checks layer dependency violations

# - DependencyValidator: Ensures crate dependencies follow rules
```

## Consequences

### Positive

- **Clear Boundaries**: Each crate has explicit responsibilities
- **Testability**: Test domain/application without infrastructure
- **Compilation**: Parallel builds, incremental compilation
- **Maintainability**: Changes isolated to appropriate layers
- **Onboarding**: Developers know where code belongs

### Negative

- **Complexity**: Seven crates vs one requires coordination
- **Boilerplate**: Port traits need implementations in multiple places
- **Learning Curve**: Clean Architecture concepts required

### Neutral

- **Cargo Workspace**: Standard Rust pattern, well-supported tooling
- **Feature Flags**: Providers can be optionally included

## Implementation Notes

### Adding a New Provider

1. Create implementation in `mcb-providers/src/<category>/`
2. Implement the port trait from `mcb-domain/src/ports/`
3. Register via `#[linkme::distributed_slice]` for auto-discovery
4. Add feature flag in `mcb-providers/Cargo.toml` if needed

### Adding a New Use Case

1. Define service interface in `mcb-domain/src/ports/` (port traits are in domain per ADR-029, superseded by ADR-050)
2. Implement service in `mcb-application/src/services/`
3. Inject port dependencies via constructor
4. Wire in `mcb-infrastructure/src/di/` if needed

### Testing Patterns

```rust
// Unit test (mcb-application)
#[tokio::test]
async fn test_search_service() {
    let embedding = Arc::new(MockEmbeddingProvider::new());
    let vector_store = Arc::new(MockVectorStoreProvider::new());
    let service = SearchService::new(embedding, vector_store);
    // Test without infrastructure
}

// Integration test (mcb-server) — uses AppContext composition root (ADR-050)
#[tokio::test]
async fn test_full_indexing_flow() {
    let app_context = init_app(config).await?;
    let service: Arc<dyn IndexingService> = app_context.indexing_service().clone();
    // Uses default providers resolved from config
}
```

## Canonical References

> **Note**: This ADR is a historical decision record. For current architecture
> details, consult the normative documents listed below.

- [ARCHITECTURE_BOUNDARIES.md](../architecture/ARCHITECTURE_BOUNDARIES.md) — Layer rules and module ownership (normative)
- [PATTERNS.md](../architecture/PATTERNS.md) — Technical patterns reference (normative)

## Related ADRs

- [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - Provider trait patterns
- [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async patterns per layer
- [ADR-003: Unified Provider Architecture](003-unified-provider-architecture.md) - Provider interface
- [ADR-003: Unified Provider Architecture & Routing](003-unified-provider-architecture.md) - mcb-providers organization
- [ADR-031: Documentation Excellence](031-documentation-excellence.md) - Documentation per crate
- [ADR-006: Code Audit and Improvements](006-code-audit-and-improvements.md) - Quality standards per layer
- [ADR-051: SeaQL + Loco.rs Platform Rebuild](051-seaql-loco-platform-rebuild.md) - mcb-server admin module
- [ADR-011: HTTP Transport](011-http-transport-request-response-pattern.md) - mcb-server transport layer
- [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI in mcb-infrastructure
- **Extended by**: [ADR-027: Architecture Evolution v0.1.3](027-architecture-evolution-v013.md) - Introduces bounded contexts within layers

## References

- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [linkme Documentation](https://docs.rs/linkme) (compile-time discovery in current DI; see ADR-050)
- Workspace-next refactoring plan (January 2026)
