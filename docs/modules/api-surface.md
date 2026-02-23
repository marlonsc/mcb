<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# API Surface Analysis

This document provides an overview of the public API surface of the Memory Context Browser.

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Facade crate | [`crates/mcb/src/lib.rs`](../../crates/mcb/src/lib.rs) — top-level re-exports |
| Domain | [`crates/mcb-domain/src/lib.rs`](../../crates/mcb-domain/src/lib.rs) · [domain.md](./domain.md) |
| Server | [`crates/mcb-server/src/lib.rs`](../../crates/mcb-server/src/lib.rs) · [server.md](./server.md) |
| Infrastructure | [`crates/mcb-infrastructure/src/lib.rs`](../../crates/mcb-infrastructure/src/lib.rs) · [infrastructure.md](./infrastructure.md) |
| Providers | [`crates/mcb-providers/src/lib.rs`](../../crates/mcb-providers/src/lib.rs) · [providers.md](./providers.md) |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-013`](../adr/013-clean-architecture-crate-separation.md) |

## Crate Public APIs

### mcb (Facade Crate)

Re-exports from internal crates for unified access. Source: `crates/mcb/src/lib.rs`.

```rust
// Domain layer re-exports (all of mcb-domain)
pub mod domain { pub use mcb_domain::*; }

// Server layer re-exports (all of mcb-server)
pub mod server { pub use mcb_server::*; }

// Infrastructure layer re-exports
pub mod infrastructure { pub use mcb_infrastructure::*; }

// Root-level convenience re-exports
pub use domain::*;           // All domain types
pub use server::run;         // Main server entry point
pub use server::{McpServer, McpServerBuilder};
```

### mcb-domain

Core types and port traits (source of truth for all domain contracts):

- **Entities**: `CodeChunk`, `Codebase`, `Project`, `Organization`, `AgentSession`, `Observation`, `Repository`
- **Value objects**: `Embedding`, `SearchResult`, `Language`
- **Errors**: `Error`, `Result<T>`
- **Port traits**: `EmbeddingProvider`, `VectorStoreProvider`, `CacheProvider`, `HybridSearchProvider`,
  `LanguageChunkingProvider`, `VcsProvider`, `CryptoProvider`
- **Repository ports**: `MemoryRepository`, `AgentRepository`, `VcsEntityRepository`, `ProjectRepository`, ...

### mcb-application

Use case services (source: `crates/mcb-application/src/use_cases/`):

- `IndexingServiceImpl` — codebase indexing and ingestion
- `SearchServiceImpl` — query processing and ranking
- `ContextServiceImpl` — embedding and vector operations
- `MemoryServiceImpl` — observation/memory use cases
- `AgentSessionServiceImpl` — agent session lifecycle
- `decorators::InstrumentedEmbeddingProvider` — metrics decorator (OCP pattern)

### mcb-server

MCP protocol server (source: `crates/mcb-server/src/`):

- `McpServer` — main server struct
- `McpServerBuilder` — builder for server configuration
- `run(config, server_mode)` — async entry point
- Admin REST API via Poem (`/admin/*` routes)

### mcb-providers

External integrations (source: `crates/mcb-providers/src/`):

- **Embedding**: `OpenAI`, `Ollama`, `VoyageAI`, `Gemini`, `FastEmbed`, `Anthropic` (6 providers)
- **Vector Store**: `EdgeVec`, `Milvus`, `Qdrant`, `Pinecone`, encrypted decorator (5 backends)
- **Cache**: `Moka`, `Redis` (2 backends)
- **Events**: Tokio broadcast, NATS (2 backends)
- **Language**: 13 languages via tree-sitter v0.26
- **Database**: SQLite + FTS5 (7 repository implementations)

### mcb-infrastructure

Configuration and DI (source: `crates/mcb-infrastructure/src/`):

- `config::AppConfig`, `ServerConfig`, `InfrastructureConfig` — typed TOML config
- `di::` — AppContext manual composition root in `bootstrap.rs` ([ADR-050](../adr/050-manual-composition-root-dill-removal.md))
- `routing::` — provider routing and selection
- `crypto::` — AES-GCM encryption
- `cache::` — Moka/Redis infrastructure
- `logging::` — structured tracing setup

### mcb-validate

Architecture validation tooling (source: `crates/mcb-validate/src/`):

- `ValidatorRegistry` — registry of all validators
- `ValidationConfig` — multi-directory scan config
- `GenericReporter` / `GenericReport` — unified violation reporting
- 18+ validators (Clean Architecture, SOLID, Quality, SSOT, ...)

## API Stability

### Current Status

- **Version**: 0.2.1
- **Stability**: Stable for documented APIs
- **Compatibility**: Semantic versioning from v0.1.0+

### Breaking Change Policy

- Minor versions (0.x.0): May include breaking changes with CHANGELOG notice
- Patch versions (0.x.y): Bug fixes and non-breaking additions only
- Major version (1.0.0+): Stable API with deprecation cycles

---

### Updated 2026-02-20 — Corrected stale API names (run_server→run, removed ChunkingOrchestrator); added bidirectional code↔docs cross-reference block; verified against actual crate exports (v0.2.1)
