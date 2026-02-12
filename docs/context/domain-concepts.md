# Domain Concepts Context

**Last updated:** 2026-02-11
**Source:** `mcb-domain/src/entities/`, `mcb-domain/src/ports/`, `mcb-domain/src/value_objects/`, `mcb-application/src/`

## Overview

MCB delivers semantic code search by combining vector embeddings, git context, agent session tracking, and MCP tooling. Multi-tenant by design — every entity carries `org_id` for row-level isolation.

## Core Entities

| Entity | Location | Purpose |
|--------|----------|---------|
| **CodeChunk** | `entities/code_chunk.rs` | Atomic unit of semantic indexing — AST-parsed code segment |
| **Project** | `entities/project.rs` | Root aggregate — registered codebase with type detection |
| **Organization** | `entities/organization.rs` | Tenant root — multi-tenant isolation boundary |
| **Repository** | `entities/repository.rs` | VCS repository with branch tracking |
| **Plan** | `entities/plan.rs` | Versioned execution plan (Draft→Active→Executing→Completed→Archived) |
| **AgentSession** | `entities/agent/session.rs` | Agent execution lifecycle with timing/metrics |
| **Observation** | `entities/observation.rs` | Memory record (Code, Decision, Context, Error, Summary, Execution, QualityGate) |
| **Workflow** | `entities/workflow.rs` | Workflow FSM for session state management |
| **Worktree** | `entities/worktree.rs` | Git worktree with agent-worktree assignments |
| **User/Team** | `entities/{user,team}.rs` | Identity and team membership |
| **Issue** | `entities/issue.rs` | Issue tracking with comments, labels |
| **ApiKey** | `entities/api_key.rs` | Authentication key entity |

## Value Objects

| Value Object | Location | Purpose |
|-------------|----------|---------|
| **Embedding** | `value_objects/embedding.rs` | Semantic vector (Vec<f32>) with model/dimensions |
| **SearchResult** | `value_objects/search.rs` | Ranked result with score (0.0-1.0), file path, content |
| **Strong-Typed IDs** | `value_objects/ids.rs` | CollectionId, ChunkId, SessionId, OrgId, etc. |
| **ProjectContext** | `value_objects/project_context.rs` | Enriched project context for search |
| **OrgContext** | `value_objects/org_context.rs` | Organization-scoped context |
| **Browse types** | `value_objects/browse/` | FileInfo, FileTreeNode, CollectionInfo, HighlightedCode |

## Port Interfaces (Domain Boundaries)

### Provider Ports (External Services)

- **EmbeddingProvider**: embed, embed_batch, dimensions, health_check
- **VectorStoreProvider**: create_collection, insert, search_similar, delete
- **HybridSearchProvider**: BM25 lexical + semantic combined search
- **LanguageChunkingProvider**: Language-specific AST parsing
- **VcsProvider**: git clone, fetch, branches, commits
- **CryptoProvider**: Encryption/decryption
- **CacheProvider**: Distributed caching with TTL
- **ProjectDetectionProvider**: Detect project type (Cargo, npm, Python, Go, Maven)

### Repository Ports (Persistence)

- **ChunkRepository**: Code chunk CRUD
- **MemoryRepository**: Observation storage + FTS search
- **AgentRepository**: Agent session persistence + query
- **ProjectRepository**: Project CRUD
- **VcsEntityRepository**: Repository/branch persistence
- **PlanEntityRepository**: Plan version/review persistence
- **IssueEntityRepository**: Issue tracking persistence
- **OrgEntityRepository**: Multi-tenant org data

### Service Ports (Business Logic)

- **IndexingServiceInterface**: Codebase indexing orchestration
- **SearchServiceInterface**: Semantic search with filters
- **ContextServiceInterface**: Context aggregation
- **ValidationServiceInterface**: Code quality validation
- **MemoryServiceInterface**: Observation management
- **AgentSessionServiceInterface**: Agent lifecycle

## Domain Events (`events/domain_events.rs`)

IndexRebuild, IndexingStarted, IndexingProgress, IndexingCompleted, SyncCompleted, CacheInvalidate, SnapshotCreated, FileChangesDetected, ServiceStateChanged

## Key Enums & State Machines

| Enum | Values |
|------|--------|
| ProjectType | Cargo, Npm, Python, Go, Maven |
| VcsType | Git, Mercurial, Svn |
| ObservationType | Code, Decision, Context, Error, Summary, Execution, QualityGate |
| PlanStatus | Draft → Active → Executing → Completed → Archived |
| OrgStatus | Active, Suspended, Archived |
| AgentSessionStatus | Running, Completed, Failed |
| WorkflowState | FSM states for session management |
| WorktreeStatus | Available, Assigned, Archived |

## Key Invariants

1. **Multi-tenant isolation**: All entities carry `org_id`; Organization is the root boundary
2. **Indexing pipeline**: Project → Repository → CodeChunk → Embedding → VectorStore
3. **Search architecture**: Semantic (embedding+vector) + Hybrid (BM25+semantic)
4. **Memory**: Observations → FTS-indexed → searchable session context
5. **Agent execution**: Session → Checkpoints → ToolCalls → Delegations (nested agents)

## Related Context

- `docs/context/technical-patterns.md` — architecture and patterns
- `docs/context/integrations.md` — external service boundaries
- `docs/context/project-state.md` — current phase and metrics

## Mirror Context

- `context/project-intelligence/domain-concepts.md` — compact operational mirror

## Change Notes

- 2026-02-11T23:26:00-03:00 - Reconciled with `context/project-intelligence/domain-concepts.md` and added mirror reference.
