<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
# domain Module

**Source**: `crates/mcb-domain/src/`
**Crate**: `mcb-domain`
**Files**: 25+
**Lines of Code**: ~2,500
**Traits**: 41 (13 provider ports + 8 repository ports + 9 service ports + 8 infrastructure ports + 3 top-level ports)
**Structs**: 20+
**Enums**: 8

**Project links**: See `docs/architecture/ARCHITECTURE.md` and `docs/developer/ROADMAP.md` for domain architecture and v0.2.1 objectives.

## Overview

The domain module defines the core business entities, value objects, and repository interfaces following Clean Architecture principles. All domain logic is technology-agnostic, with external concerns abstracted behind port traits. Multi-tenant by design — every entity carries `org_id` for row-level isolation.

MCB delivers semantic code search by combining vector embeddings, git context, agent session tracking, and MCP tooling. Organization is the root tenant boundary.

> **Note**: Port traits (EmbeddingProvider, VectorStoreProvider, etc.) are defined in `mcb-domain/src/ports/`. The domain layer includes entities, value objects, and all port trait boundaries.

## Core Entities (`entities/`)

| Entity | File | Purpose |
| -------- | ------ | --------- |
| **CodeChunk** | `code_chunk.rs` | Atomic unit of semantic indexing — AST-parsed code segment with metadata |
| **Codebase** | `codebase.rs` | Repository metadata container |
| **Project** | `project.rs` | Root aggregate — registered codebase with type detection (Cargo, npm, Python, Go, Maven) |
| **Organization** | `organization.rs` | Tenant root — multi-tenant isolation boundary, carries `org_id` |
| **Repository** | `repository.rs` | VCS repository with branch tracking |
| **Plan** | `plan.rs` | Versioned execution plan (Draft → Active → Executing → Completed → Archived) |
| **AgentSession** | `agent/session.rs` | Agent execution lifecycle with timing, metrics, and checkpoints |
| **Observation** | `observation.rs` | Memory record (Code, Decision, Context, Error, Summary, Execution, QualityGate) |
| **Workflow** | `workflow.rs` | Workflow FSM for session state management |
| **Worktree** | `worktree.rs` | Git worktree with agent-worktree assignments (Available, Assigned, Archived) |
| **User** | `user.rs` | Identity entity |
| **Team** | `team.rs` | Team membership entity |
| **Issue** | `issue.rs` | Issue tracking with comments, labels |
| **ApiKey** | `api_key.rs` | Authentication key entity |
| **Submodule** | `submodule.rs` | Git submodule metadata |

## Value Objects (`value_objects/`)

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

## Repository Interfaces (`repositories/`)

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

### Provider Ports (`mcb-domain/src/ports/providers/` — External Services)

| Port | Operations | Implementations |
| ------ | ----------- | ---------------- |
| `EmbeddingProvider` | `embed`, `embed_batch`, `dimensions`, `health_check` | OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic |
| `VectorStoreProvider` | `create_collection`, `insert`, `search_similar`, `delete` | EdgeVec, Milvus, Qdrant, Pinecone, Encrypted |
| `HybridSearchProvider` | BM25 lexical + semantic combined search | Composite implementation |
| `LanguageChunkingProvider` | Language-specific AST parsing | 13 tree-sitter processors |
| `VcsProvider` | `clone`, `fetch`, `branches`, `commits`, `diffs` | git2 v0.20 |
| `CryptoProvider` | Encryption/decryption | AES-256-GCM, Argon2 |
| `CacheProvider` | Distributed caching with TTL | Moka, Redis |
| `ProjectDetectionProvider` | Detect project type from manifests | Cargo, npm, Python, Go, Maven |

### Repository Ports (`mcb-domain` — Persistence)

| Port | Purpose | Implementation Location |
| ------ | --------- | ------------------------ |
| `ChunkRepository` | Code chunk CRUD | `mcb-providers/` |
| `MemoryRepository` | Observation storage + FTS search | `mcb-providers/src/database/sqlite/` |
| `AgentRepository` | Agent session persistence + query | `mcb-providers/src/database/sqlite/` |
| `ProjectRepository` | Project CRUD | `mcb-providers/src/database/sqlite/` |
| `VcsEntityRepository` | Repository/branch persistence | `mcb-providers/src/database/sqlite/` |
| `PlanEntityRepository` | Plan version/review persistence | `mcb-providers/src/database/sqlite/` |
| `IssueEntityRepository` | Issue tracking persistence | `mcb-providers/src/database/sqlite/` |
| `OrgEntityRepository` | Multi-tenant org data | `mcb-providers/src/database/sqlite/` |

### Service Ports (`mcb-domain/src/ports/services/` — Business Logic)

| Port | Purpose |
| ------ | --------- |
| `IndexingServiceInterface` | Codebase indexing orchestration |
| `BatchIndexingServiceInterface` | Batch indexing orchestration |
| `SearchServiceInterface` | Semantic search with filters |
| `ContextServiceInterface` | Context aggregation |
| `ValidationServiceInterface` | Code quality validation (12 rules) |
| `MemoryServiceInterface` | Observation management |
| `AgentSessionServiceInterface` | Agent lifecycle management |
| `ProjectDetectorService` | Project type detection orchestration |
| `FileHashService` | File hashing service boundary |
| `ChunkingOrchestratorInterface` | Chunking orchestration boundary |
| `CodeChunker` | Language chunking service boundary |

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
├── repositories/               # Repository port traits
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

---

### Updated 2026-02-12 — Enriched with full entity catalog, port interfaces, state machines, and domain invariants (v0.2.1)
