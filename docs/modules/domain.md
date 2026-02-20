<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# domain Module

**Source**: `crates/mcb-domain/src/`
**Crate**: `mcb-domain`
**Files**: 25+
**Lines of Code**: ~2,500
**Traits**: 41 (13 provider ports + 8 repository ports + 9 service ports + 8 infrastructure ports + 3 top-level ports)
**Structs**: 20+
**Enums**: 8

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-domain/src/lib.rs`](../../crates/mcb-domain/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-domain/src/lib.rs`](../../crates/mcb-domain/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-001`](../adr/001-modular-crates-architecture.md) · [`ADR-013`](../adr/013-clean-architecture-crate-separation.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The domain module defines the core business entities, value objects, and repository interfaces following Clean Architecture principles. All domain logic is technology-agnostic, with external concerns abstracted behind port traits. Multi-tenant by design — every entity carries `org_id` for row-level isolation.

MCB delivers semantic code search by combining vector embeddings, git context, agent session tracking, and MCP tooling. Organization is the root tenant boundary.

> **Note**: Port traits (EmbeddingProvider, VectorStoreProvider, etc.) are defined in `mcb-domain/src/ports/`. The domain layer includes entities, value objects, and all port trait boundaries.

## Core Entities

| Entity | File | Purpose |
| -------- | ------ | --------- |
| **CodeChunk** | [`code_chunk.rs`](../../crates/mcb-domain/src/entities/code_chunk.rs) | Atomic unit of semantic indexing — AST-parsed code segment with metadata |
| **Codebase** | [`codebase.rs`](../../crates/mcb-domain/src/entities/codebase.rs) | Repository metadata container |
| **Project** | [`project.rs`](../../crates/mcb-domain/src/entities/project.rs) | Root aggregate — registered codebase with type detection |
| **Organization** | [`organization.rs`](../../crates/mcb-domain/src/entities/organization.rs) | Tenant root — multi-tenant isolation boundary, carries `org_id` |
| **Repository** | [`repository.rs`](../../crates/mcb-domain/src/entities/repository.rs) | VCS repository with branch tracking |
| **Plan** | [`plan.rs`](../../crates/mcb-domain/src/entities/plan.rs) | Versioned execution plan (Draft → Active → Executing → Completed) |
| **AgentSession** | [`agent/session.rs`](../../crates/mcb-domain/src/entities/agent/session.rs) | Agent execution lifecycle with timing and metrics |
| **Observation** | [`observation.rs`](../../crates/mcb-domain/src/entities/observation.rs) | Memory record (Code, Decision, Context, Error, etc.) |
| **Workflow** | [`workflow.rs`](../../crates/mcb-domain/src/entities/workflow.rs) | Workflow FSM for session state management |
| **Worktree** | [`worktree.rs`](../../crates/mcb-domain/src/entities/worktree.rs) | Git worktree with agent-worktree assignments |
| **User** | [`user.rs`](../../crates/mcb-domain/src/entities/user.rs) | Identity entity |
| **Team** | [`team.rs`](../../crates/mcb-domain/src/entities/team.rs) | Team membership entity |
| **Issue** | [`issue.rs`](../../crates/mcb-domain/src/entities/issue.rs) | Issue tracking with comments, labels |
| **ApiKey** | [`api_key.rs`](../../crates/mcb-domain/src/entities/api_key.rs) | Authentication key entity |
| **Submodule** | [`submodule.rs`](../../crates/mcb-domain/src/entities/submodule.rs) | Git submodule metadata |

## Value Objects

| Value Object | File | Purpose |
| ------------- | ------ | --------- |
| **Embedding** | `embedding.rs` | Semantic vector (`Vec<f32>`) with model name and dimensions |
| **SearchResult** | `search.rs` | Ranked result with score (0.0–1.0), file path, content snippet |
| **Strong-Typed IDs** | `ids.rs` | `CollectionId`, `ChunkId`, `SessionId`, `OrgId`, etc. |
| **ProjectContext** | `project_context.rs` | Enriched project context for search queries |
| **OrgContext** | `org_context.rs` | Organization-scoped context |
| **Browse types** | `browse/` | `FileInfo`, `FileTreeNode`, `CollectionInfo`, `HighlightedCode` |
| **Config** | `config.rs` | Configuration value objects |
| **Types** | `types.rs` | Shared primitive type aliases |

## Repository Interfaces

| Port | File | Purpose |
| ------ | ------ | --------- |
| `ChunkRepository` | `chunk_repository.rs` | Code chunk CRUD (`Send + Sync`; DI via dill, ADR-029) |
| `SearchRepository` | `search_repository.rs` | Search operations (`Send + Sync`; DI via dill, ADR-029) |

## Domain Events (`events/domain_events.rs`)

Events published through the `EventPublisher` interface:

- `IndexRebuild` — Full re-index requested
- `IndexingStarted` / `IndexingProgress` / `IndexingCompleted` — Indexing lifecycle
- `SyncCompleted` — VCS sync finished
- `CacheInvalidate` — Cache eviction signal
- `SnapshotCreated` — Point-in-time snapshot taken
- `FileChangesDetected` — Filesystem watcher trigger
- `ServiceStateChanged` — Service lifecycle state transitions

## Port Interfaces (Domain Boundaries)

<a name="provider-ports"></a>
### Provider Ports

| Port | Operations | Implementations |
| ------ | ----------- | ---------------- |
| [`EmbeddingProvider`](../../crates/mcb-domain/src/ports/providers/embedding.rs) | `embed`, `embed_batch`, `dimensions` | OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic |
| [`VectorStoreProvider`](../../crates/mcb-domain/src/ports/providers/vector_store/provider.rs) | `create_collection`, `insert`, `search` | EdgeVec, Milvus, Qdrant, Pinecone, Encrypted |
| [`HybridSearchProvider`](../../crates/mcb-domain/src/ports/providers/hybrid_search.rs) | BM25 lexical + semantic search | Composite implementation |
| [`LanguageChunkingProvider`](../../crates/mcb-domain/src/ports/providers/language_chunking.rs) | Language-specific AST parsing | 13 tree-sitter processors |
| [`VcsProvider`](../../crates/mcb-domain/src/ports/providers/vcs.rs) | `clone`, `fetch`, `branches`, `commits` | git2 v0.20 |
| [`CryptoProvider`](../../crates/mcb-domain/src/ports/providers/crypto.rs) | Encryption/decryption | AES-256-GCM, Argon2 |
| [`CacheProvider`](../../crates/mcb-domain/src/ports/providers/cache/provider.rs) | Distributed caching with TTL | Moka, Redis |
| [`ProjectDetectionProvider`](../../crates/mcb-domain/src/ports/providers/project_detection.rs) | Detect project type from manifests | Cargo, npm, Python, Go, Maven |

<a name="repository-ports"></a>
### Repository Ports

| Port | Purpose | Implementation Location |
| ------ | --------- | ------------------------ |
| `ChunkRepository` | Persistence of AST-parsed code chunks and search statistics | [`mcb-providers`](../../crates/mcb-providers/src/lib.rs) |
| `MemoryRepository` | Multi-tenant observation storage with FTS5 lexical search capabilities | [`memory_repository.rs`](../../crates/mcb-providers/src/database/sqlite/memory_repository.rs) |
| `AgentRepository` | Composite management of **Agent Sessions**, **Delegations**, **Tool Calls**, and **Checkpoints** | [`agent_repository.rs`](../../crates/mcb-providers/src/database/sqlite/agent_repository.rs) |
| `ProjectRepository` | Persistence of **Project** root entities (multi-tenant boundary) | [`project_repository.rs`](../../crates/mcb-providers/src/database/sqlite/project_repository.rs) |
| `VcsEntityRepository` | Composite management of **Repositories**, **Branches**, **Worktrees**, and **Agent-Worktree Assignments** | [`vcs_entity_repository.rs`](../../crates/mcb-providers/src/database/sqlite/vcs_entity_repository.rs) |
| `PlanEntityRepository` | Persistence for **Plans**, **Versions**, and **Reviews** (Execution planning) | [`plan_entity_repository.rs`](../../crates/mcb-providers/src/database/sqlite/plan_entity_repository.rs) |
| `IssueEntityRepository` | Composite management of **Issues**, **Comments**, **Labels**, and **Label Assignments** | [`issue_entity_repository.rs`](../../crates/mcb-providers/src/database/sqlite/issue_entity_repository.rs) |
| `OrgEntityRepository` | Composite management of **Organizations**, **Users**, **Teams**, and **API Keys** | [`org_entity_repository.rs`](../../crates/mcb-providers/src/database/sqlite/org_entity_repository.rs) |

<a name="service-ports"></a>
### Service Ports

| Port | Purpose |
| ------ | --------- |
| [`IndexingServiceInterface`](../../crates/mcb-domain/src/ports/services/indexing.rs) | Codebase indexing orchestration |
| [`BatchIndexingServiceInterface`](../../crates/mcb-domain/src/ports/services/indexing.rs) | Batch indexing orchestration |
| [`SearchServiceInterface`](../../crates/mcb-domain/src/ports/services/search.rs) | Semantic search with filters |
| [`ContextServiceInterface`](../../crates/mcb-domain/src/ports/services/context.rs) | Context aggregation |
| [`ValidationServiceInterface`](../../crates/mcb-domain/src/ports/services/validation.rs) | Code quality validation (12 rules) |
| [`MemoryServiceInterface`](../../crates/mcb-domain/src/ports/services/memory.rs) | Observation management |
| [`AgentSessionServiceInterface`](../../crates/mcb-domain/src/ports/services/agent.rs) | Agent lifecycle management |
| [`ProjectDetectorService`](../../crates/mcb-domain/src/ports/services/project.rs) | Project type detection orchestration |
| [`FileHashService`](../../crates/mcb-domain/src/ports/services/hash.rs) | File hashing service boundary |
| [`ChunkingOrchestratorInterface`](../../crates/mcb-domain/src/ports/services/chunking.rs) | Chunking orchestration boundary |
| [`CodeChunker`](../../crates/mcb-domain/src/ports/services/chunking.rs) | Language chunking service boundary |

## Key Enums & State Machines

| Enum | Values |
| ------ | -------- |
| `ProjectType` | Cargo, Npm, Python, Go, Maven |
| `VcsType` | Git, Mercurial, Svn |
| `ObservationType` | Code, Decision, Context, Error, Summary, Execution, QualityGate |
| `PlanStatus` | Draft → Active → Executing → Completed → Archived |
| `OrgStatus` | Active, Suspended, Archived |
| `AgentSessionStatus` | Running, Completed, Failed |
| `WorkflowState` | FSM states for session management |
| `WorktreeStatus` | Available, Assigned, Archived |

## Domain Invariants

1. **Multi-tenant isolation**: All entities carry `org_id`; Organization is the root boundary
2. **Indexing pipeline**: Project → Repository → CodeChunk → Embedding → VectorStore
3. **Search architecture**: Semantic (embedding + vector) + Hybrid (BM25 + semantic)
4. **Memory model**: Observations → FTS-indexed → searchable session context
5. **Agent execution**: Session → Checkpoints → ToolCalls → Delegations (nested agents)

## File Structure (Actual)

```text
crates/mcb-domain/src/
├── entities/                   # Domain entities
│   ├── agent/                  # Agent-related entities
│   │   └── session.rs
│   ├── memory/                 # Memory-related entities
│   ├── vcs/                    # VCS-related entities
│   ├── api_key.rs
│   ├── code_chunk.rs
│   ├── codebase.rs
│   ├── issue.rs
│   ├── observation.rs
│   ├── organization.rs
│   ├── plan.rs
│   ├── project.rs
│   ├── repository.rs
│   ├── submodule.rs
│   ├── team.rs
│   ├── user.rs
│   ├── workflow.rs
│   ├── worktree.rs
│   └── mod.rs
├── events/                     # Domain events
│   ├── domain_events.rs
│   └── mod.rs
├── ports/                      # Port trait boundaries (Clean Architecture)
│   ├── providers/              # External service port traits
│   │   ├── analysis.rs         # Code analysis provider
│   │   ├── cache.rs            # Cache provider
│   │   ├── config.rs           # Config provider
│   │   ├── crypto.rs           # Crypto provider
│   │   ├── embedding.rs        # Embedding provider
│   │   ├── http.rs             # HTTP client provider
│   │   ├── hybrid_search.rs    # Hybrid search provider
│   │   ├── language_chunking.rs # Language chunking provider
│   │   ├── metrics.rs          # Metrics provider
│   │   ├── metrics_analysis.rs # Metrics analysis provider
│   │   ├── project_detection.rs # Project detection provider
│   │   ├── validation.rs       # Validation provider
│   │   ├── vcs.rs              # VCS provider
│   │   ├── vector_store.rs     # Vector store provider
│   │   └── mod.rs
│   ├── repositories/           # Persistence port traits
│   │   ├── agent_repository.rs
│   │   ├── file_hash_repository.rs
│   │   ├── issue_entity_repository.rs
│   │   ├── memory_repository.rs
│   │   ├── org_entity_repository.rs
│   │   ├── plan_entity_repository.rs
│   │   ├── project_repository.rs
│   │   ├── vcs_entity_repository.rs
│   │   └── mod.rs
│   ├── services/               # Business logic port traits
│   │   ├── agent.rs
│   │   ├── chunking.rs
│   │   ├── context.rs
│   │   ├── hash.rs
│   │   ├── indexing.rs
│   │   ├── memory.rs
│   │   ├── project.rs
│   │   ├── search.rs
│   │   ├── validation.rs
│   │   └── mod.rs
│   ├── infrastructure/         # Infrastructure port traits
│   │   ├── auth.rs
│   │   ├── database.rs
│   │   ├── events.rs
│   │   ├── metrics.rs
│   │   ├── routing.rs
│   │   ├── snapshot.rs
│   │   ├── state_store.rs
│   │   ├── sync.rs
│   │   └── mod.rs
│   ├── admin.rs                # Admin port traits
│   ├── browse.rs               # Browse port traits
│   ├── jobs.rs                 # Job scheduling port traits
│   └── mod.rs
├── repositories/               # Legacy repository port traits
│   ├── chunk_repository.rs
│   ├── search_repository.rs
│   └── mod.rs
├── value_objects/              # Value objects
│   ├── browse/                # Browse-related VOs
│   ├── config.rs
│   ├── embedding.rs
│   ├── ids.rs                 # Strong-typed IDs
│   ├── org_context.rs
│   ├── project_context.rs
│   ├── search.rs
│   ├── types.rs
│   └── mod.rs
├── constants.rs                # Domain constants
├── error.rs                    # Domain error types
└── mod.rs                      # Module exports
```

## Domain Utilities

| Utility | File | Purpose |
| ------- | ---- | ------- |
| **Analysis** | [`analysis.rs`](../../crates/mcb-domain/src/utils/analysis.rs) | Domain-specific analysis helpers (Regex, string processing) |
| **Common** | [`common.rs`](../../crates/mcb-domain/src/utils/mod.rs) | Shared domain utilities |

## Testing Utilities

| Utility | File | Purpose |
| ------- | ---- | ------- |
| **Test Utils** | [`test_utils.rs`](../../crates/mcb-domain/src/test_utils.rs) | Shared domain specimen creation (Projects, Phases, Agents) |

---

### Updated 2026-02-20 — Consolidated SSOT and traceability (v0.2.1)
