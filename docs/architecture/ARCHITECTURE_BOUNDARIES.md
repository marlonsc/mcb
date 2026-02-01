# Architecture Boundaries - Layer Rules and Module Ownership

**Version**: v0.1.5
**Status**: Baseline Documentation
**Last Updated**: 2026-01-28

This document defines the strict architectural boundaries for the MCB (MCP Context Browser) project following Clean Architecture principles with 8 workspace crates.

---

## Table of Contents

1.  [Crate Structure](#crate-structure)
2.  [Layer Dependency Rules](#layer-dependency-rules)
3.  [Port/Adapter Pattern](#portadapter-pattern)
4.  [Module Ownership](#module-ownership)
5.  [Boundary Violations](#boundary-violations)
6.  [Validation Rules](#validation-rules)

---

## Crate Structure

MCB follows a layered architecture across 8 Cargo workspace crates:

```
crates/
├── mcb/                 # Facade (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services, registry
├── mcb-providers/       # Layer 3: Provider implementations
├── mcb-infrastructure/  # Layer 4: DI, config, health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
├── mcb-validate/        # Dev tooling: architecture validation
└── (tests/)             # Integration and golden tests
```

### Dependency Direction (Inward Only)

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓                    ↑
              mcb-providers ─────────────┘
```

**Critical Rule**: Dependencies ALWAYS point inward. Outer layers depend on inner layers, never the reverse.

---

## Layer Dependency Rules

### Layer 1: mcb-domain (Core)

**Purpose**: Domain entities, value objects, port traits, domain errors

**Allowed Dependencies**:

-   Standard library only
-   `thiserror` for error types
-   `serde` for serialization (optional feature)

**Prohibited Dependencies**:

-   NO dependencies on other MCB crates
-   NO infrastructure concerns (HTTP, database, filesystem)
-   NO concrete implementations (only trait definitions)

**Exports**:

-   Entities: `CodeChunk`, `Embedding`, `SearchResult`, `ChunkMetadata`
-   Value objects: `Vector`, `Distance`, `Score`
-   Port traits: `EmbeddingProvider`, `VectorStoreProvider`, `CacheProvider`, `LanguageChunkingProvider`
-   Domain errors: `DomainError`, `ValidationError`

**Module Structure**:

```
mcb-domain/src/
├── entities/           # Domain entities
├── value_objects/      # Value objects
├── ports/              # Port traits (interfaces)
│   └── providers/      # Provider traits
└── errors/             # Domain errors
```

---

### Layer 2: mcb-application (Use Cases)

**Purpose**: Application services, use cases, business logic orchestration

**Allowed Dependencies**:

-   `mcb-domain` (ports, entities, errors)
-   `async-trait` for async traits
-   `tokio` for async runtime
-   `linkme` for provider registration

**Prohibited Dependencies**:

-   NO direct dependency on `mcb-providers` (use ports from mcb-domain)
-   NO direct dependency on `mcb-infrastructure` (use DI)
-   NO HTTP/transport concerns

**Exports**:

-   Services: `ContextService`, `SearchService`, `IndexingService`
-   Registry: `EMBEDDING_PROVIDERS`, `VECTOR_STORE_PROVIDERS` (linkme slices)
-   Admin ports: `IndexingOperationsInterface`, `PerformanceMetricsInterface`
-   Infrastructure ports: `EventBusProvider`, `AuthServiceInterface`

**Module Structure**:

```
mcb-application/src/
├── use_cases/          # Application services
│   ├── context_service.rs
│   ├── search_service.rs
│   └── indexing_service.rs
├── ports/              # Application-level ports
│   ├── admin/          # Admin operation interfaces
│   └── registry/       # Provider registry (linkme)
└── errors/             # Application errors
```

**Registry Pattern** (linkme):

```rust
// Declare slice in mcb-application
#[linkme::distributed_slice]
pub static EMBEDDING_PROVIDERS: [EmbeddingProviderEntry] = [..];

// Register in mcb-providers
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    factory: ollama_factory,  // Function pointer
};
```

---

### Layer 3: mcb-providers (Adapters)

**Purpose**: Concrete implementations of port traits

**Allowed Dependencies**:

-   `mcb-domain` (implement port traits)
-   `mcb-application` (register in linkme slices)
-   External provider SDKs (OpenAI, Ollama, Milvus, etc.)
-   `linkme` for auto-registration

**Prohibited Dependencies**:

-   NO dependency on `mcb-infrastructure` (providers are pure adapters)
-   NO dependency on `mcb-server` (transport-agnostic)

**Exports**:

-   Embedding providers: `OllamaProvider`, `OpenAIProvider`, `VoyageAIProvider`, etc.
-   Vector store providers: `MilvusProvider`, `InMemoryProvider`, `EncryptedProvider`, etc.
-   Cache providers: `MokaProvider`, `RedisProvider`, `NullProvider`
-   Language parsers: `RustChunker`, `PythonChunker`, etc.

**Module Structure**:

```
mcb-providers/src/
├── embedding/          # Embedding provider implementations
│   ├── ollama.rs
│   ├── openai.rs
│   └── voyageai.rs
├── vector_store/       # Vector store implementations
│   ├── milvus.rs
│   ├── in_memory.rs
│   └── encrypted.rs
├── cache/              # Cache implementations
└── language/           # Language-specific chunkers
```

**Registration Pattern**:

-   Each provider MUST register via `#[linkme::distributed_slice]`
-   Factory functions MUST be function pointers, NOT closures
-   Feature flags MUST gate optional providers

---

### Layer 4: mcb-infrastructure (Infrastructure)

**Purpose**: Cross-cutting concerns (DI, config, health, logging, metrics)

**Allowed Dependencies**:

-   `mcb-domain` (port traits for DI)
-   `mcb-application` (services for DI composition)
-   `mcb-providers` (concrete implementations for DI)
-   `dill` for IoC container (ADR-029)
-   `figment` for configuration (ADR-025)
-   Infrastructure libraries (tracing, metrics, etc.)

**Prohibited Dependencies**:

-   NO dependency on `mcb-server` (infrastructure is transport-agnostic)

**Exports**:

-   DI: `Catalog`, `build_catalog()`, `get_service<T>()`
-   Config: `AppConfig`, `load_config()`
-   Handles: `EmbeddingProviderHandle`, `VectorStoreProviderHandle`
-   Admin services: `EmbeddingAdminService`, `VectorStoreAdminService`
-   Health: `HealthChecker`
-   Metrics: `MetricsCollector`
-   Lifecycle: `ServiceManager`, `ShutdownCoordinator`

**Module Structure**:

```
mcb-infrastructure/src/
├── di/                 # Dependency injection (dill)
│   ├── catalog.rs      # IoC container
│   └── resolvers.rs    # Service resolution
├── config/             # Configuration (Figment)
│   ├── loader.rs
│   └── types/
├── handles/            # RwLock wrappers for runtime switching
├── admin/              # Admin services (runtime provider switching)
├── health/             # Health checking
├── metrics/            # Prometheus metrics
└── lifecycle/          # Service lifecycle management
```

**DI Pattern** (ADR-029):

```rust
// Build catalog
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        .add_value(config)
        .add_value(embedding_provider)    // From linkme registry
        .add_value(embedding_handle)      // RwLock wrapper
        .add_value(embedding_admin)       // Runtime switching
        .build()
}

// Retrieve service
pub fn get_service<T: ?Sized + Send + Sync>(catalog: &Catalog) -> Result<Arc<T>> {
    catalog.get_one::<T>()
}
```

---

### Layer 5: mcb-server (Server/Transport)

**Purpose**: MCP protocol implementation, HTTP/stdio transport, tool handlers

**Allowed Dependencies**:

-   `mcb-domain` (entities, errors)
-   `mcb-application` (services via DI)
-   `mcb-infrastructure` (DI catalog, config, health)
-   MCP libraries
-   HTTP libraries (Rocket)

**Prohibited Dependencies**:

-   NO direct use of `mcb-providers` (access via DI and port traits)

**Exports**:

-   MCP server: `MCPServer`
-   Transport: `HttpTransport`, `StdioTransport`
-   Handlers: `index_codebase`, `search_code`, `get_indexing_status`, `clear_index`

**Module Structure**:

```
mcb-server/src/
├── mcp_server.rs       # MCP server core
├── transport/          # Transport implementations
│   ├── http.rs         # HTTP transport (Rocket)
│   └── stdio.rs        # Stdio transport
├── handlers/           # MCP tool handlers
│   ├── index.rs
│   ├── search.rs
│   ├── status.rs
│   └── clear.rs
└── session/            # Session management
```

---

### Facade: mcb (Public API)

**Purpose**: Re-export public API for library users

**Allowed Dependencies**:

-   All MCB crates (selectively re-exports)

**Exports**:

-   Public entities from `mcb-domain`
-   Public services from `mcb-application`
-   Public config from `mcb-infrastructure`
-   Binary entry point in `src/main.rs`

---

### Tooling: mcb-validate (Development)

**Purpose**: Architecture validation, lint rules, quality checks

**Allowed Dependencies**:

-   All MCB crates (for analysis)
-   `tree-sitter` for AST parsing
-   Validation libraries

**Prohibited in Production**:

-   Only used in development/CI
-   NOT a runtime dependency

---

## Port/Adapter Pattern

### Port Definition (mcb-domain)

```rust
// Port trait in mcb-domain/src/ports/providers/embedding.rs
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError>;
    fn dimension(&self) -> usize;
}
```

### Adapter Implementation (mcb-providers)

```rust
// Adapter in mcb-providers/src/embedding/ollama.rs
pub struct OllamaProvider { /* ... */ }

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        // Concrete implementation
    }
    fn dimension(&self) -> usize { 768 }
}

// Auto-registration via linkme
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    factory: ollama_factory,
};
```

### Usage via DI (mcb-application)

```rust
// Service in mcb-application uses port trait
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,  // Trait object
}

impl ContextService {
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }
}
```

---

## Module Ownership

### Ownership Rules

1.  **Single Owner**: Each module has exactly one owning crate
2.  **No Duplication**: Types/traits defined once, re-exported if needed
3.  **Clear Boundaries**: Cross-crate imports are explicit and justified

### Ownership Map

| Concept | Owner | Importers |
|---------|-------|-----------|
| Port traits | `mcb-domain` | `mcb-application`, `mcb-providers` |
| Domain entities | `mcb-domain` | All layers |
| Services | `mcb-application` | `mcb-infrastructure`, `mcb-server` |
| Providers | `mcb-providers` | `mcb-infrastructure` (via DI) |
| DI container | `mcb-infrastructure` | `mcb-server` |
| Config types | `mcb-infrastructure` | `mcb-server` |
| MCP handlers | `mcb-server` | None (entry point) |

---

## Boundary Violations

### Common Violations Detected by mcb-validate

**CA001**: Layer Dependency Violation

-   **Example**: `mcb-domain` importing from `mcb-application`
-   **Fix**: Move shared code to domain, or use dependency inversion

**CA002**: Circular Dependency

-   **Example**: `mcb-application` → `mcb-infrastructure` → `mcb-application`
-   **Fix**: Extract interface to domain, use DI

**CA004**: Missing Entity ID

-   **Example**: Entity without `id` or `uuid` field
-   **Fix**: Add unique identifier field

**CA007**: Port Duplication

-   **Example**: Port trait defined in both `mcb-domain` and `mcb-application`
-   **Fix**: Define once in `mcb-domain`, import in `mcb-application`

**CA008**: Admin Service Typing

-   **Example**: Admin service using `Arc<ConcreteType>` instead of `Arc<dyn Trait>`
-   **Fix**: Use trait objects for runtime polymorphism

**LAYER002**: Cross-Layer Import Violation

-   **Example**: `mcb-domain` importing from `mcb-infrastructure`
-   **Fix**: Reverse dependency via ports/DI

---

## Validation Rules

### Automated Checks (mcb-validate)

```bash
# Run architecture validation
make validate

# Expected output
Architecture validation: 0 violations
```

### Phase-Based Validation

**Phase 1**: Dependency Graph Analysis

-   Verify layer dependency direction
-   Detect circular dependencies
-   Check crate isolation

**Phase 2**: Import Analysis

-   Validate cross-crate imports
-   Check port usage patterns
-   Verify no implementation leakage

**Phase 3**: Type Analysis

-   Check entity structure (CA004)
-   Verify port trait locations (CA007)
-   Validate admin service types (CA008)

**Phase 4**: Quality Analysis

-   Visibility rules (VIS001: `pub` vs `pub(crate)`)
-   Code quality (QUAL020: unjustified `#[allow(dead_code)]`)
-   Documentation (DOC002: missing struct docs)

**Phase 5**: Integration Validation

-   Verify linkme registration
-   Check DI catalog composition
-   Validate config loading

**Phase 6**: Metrics Validation

-   Verify expected provider counts
-   Check language support
-   Validate test coverage

**Phase 7**: YAML Rule Engine

-   Custom project-specific rules
-   Extensible validation framework

---

## Enforcement

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
- name: Validate Architecture
  run: make validate

# Fails if any violations detected
```

### Pre-Commit Hook

```bash
# .git/hooks/pre-commit
make validate QUICK=1  # Fast validation
```

### Quality Gate

**Zero tolerance**: All violations MUST be fixed before merge.

---

## References

-   **ADR-001**: Modular Crates Architecture
-   **ADR-002**: Async-First Architecture
-   **ADR-013**: Clean Architecture Crate Separation
-   **ADR-023**: Inventory to Linkme Migration
-   **ADR-024**: Handle-Based Dependency Injection (deprecated → ADR-029)
-   **ADR-025**: Figment Configuration Loading
-   **ADR-027**: Architecture Evolution v0.1.3
-   **ADR-029**: Hexagonal Architecture with dill

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.1.5 | 2026-01-28 | Baseline documentation for architecture boundaries |

---

**Maintained by**: Architecture Team
**Review Cycle**: Each major version release
**Validation Tool**: `mcb-validate` crate
