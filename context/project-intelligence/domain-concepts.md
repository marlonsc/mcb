# Domain Concepts

Last updated: 2026-02-15 (America/Sao_Paulo)

## Product Domain

MCB gives AI coding agents persistent memory, semantic code search, and codebase understanding through the MCP protocol. Core subdomains: **indexing**, **search**, **memory**, **sessions**, **validation**, **VCS context**, **project workflow**.

## Entity Catalog (`mcb-domain/src/entities/`)

### Code Intelligence

| Entity | Purpose |
|--------|---------|
| `CodeChunk` | Semantically meaningful code segment (file, range, language, metadata) |
| `CodebaseSnapshot` | Complete codebase state at a point in time |
| `FileSnapshot` | Single file state for change tracking |
| `ProjectType` | Detected project type (Cargo, npm, Python, Go, Maven) |
| `SubmoduleInfo` | VCS submodule with parent linking |

### Memory & Observations

| Entity | Purpose |
|--------|---------|
| `Observation` | Cross-session memory record (content, type, tags, metadata, content_hash) |
| `ObservationType` | Enum: `CodePattern`, `UserPreference`, `ProjectConvention`, `ErrorResolution`, `General` |
| `ObservationMetadata` | Origin context (session, agent, tool) |
| `ErrorPattern` / `ErrorPatternMatch` | Learned error patterns for prevention |
| `QualityGateResult` / `QualityGateStatus` | Quality gate outcomes |
| `SessionSummary` | Compressed session history |
| `ExecutionMetadata` / `ExecutionType` | Execution tracking for memory operations |

### Agent & Sessions

| Entity | Purpose |
|--------|---------|
| `AgentSession` | Agent session lifecycle (status, checkpoints, delegations) |
| `AgentSessionStatus` | Enum: `Active`, `Completed`, `Failed`, `Paused` |
| `AgentType` | Enum of agent types |
| `Checkpoint` / `CheckpointType` | Session save-points |
| `Delegation` / `ToolCall` | Delegation and tool call tracking |
| `WorkflowSession` / `WorkflowState` | FSM-driven session states (ADR-034) |
| `Transition` / `TransitionTrigger` | State machine transitions |

### Project Management (Multi-tenant)

| Entity | Purpose |
|--------|---------|
| `Organization` / `OrgStatus` | Top-level tenant |
| `Team` / `TeamMember` / `TeamMemberRole` | Team structure |
| `User` / `UserRole` | User identity |
| `Project` / `DetectedProject` | Project with phases, issues, decisions |
| `ProjectPhase` / `PhaseStatus` | Lifecycle phases |
| `ProjectIssue` / `IssueStatus` / `IssueType` | Issue tracking |
| `ProjectDecision` / `ProjectDependency` | Architectural decisions and deps |
| `Plan` / `PlanVersion` / `PlanReview` | Plan lifecycle with reviews |
| `IssueComment` / `IssueLabel` | Issue details |

### VCS

| Entity | Purpose |
|--------|---------|
| `VcsRepository` / `RepositoryId` | Git repository |
| `VcsBranch` / `VcsCommit` | Branch and commit history |
| `RefDiff` / `FileDiff` / `DiffStatus` | Branch comparison |
| `Repository` / `Branch` / `VcsType` | Persisted CRUD entities |
| `Worktree` / `WorktreeStatus` / `AgentWorktreeAssignment` | Agent worktree allocation |

## Port Traits (Contracts) — `mcb-domain/src/ports/`

| Category | Key Traits |
|----------|-----------|
| **Providers** | `EmbeddingProvider`, `VectorStoreProvider`, `CacheProvider`, `LanguageChunkingProvider`, `HybridSearchProvider`, `ValidationProvider`, `MetricsAnalysisProvider` |
| **Repositories** | `AgentRepository`, `MemoryRepository` + entity-specific repos |
| **Infrastructure** | `DatabaseExecutor`, `EventBusProvider`, `SyncCoordinator`, `SnapshotProvider`, `StateStoreProvider`, `AuthServiceInterface` |
| **Services** | `ValidationServiceInterface`, `BrowseServiceInterface`, `HighlightServiceInterface` |
| **Admin** | `IndexingOperationsInterface`, `PerformanceMetricsInterface`, `ShutdownCoordinator` |
| **Jobs** | `JobManagerInterface` (background task lifecycle) |

## Error Model (`mcb-domain/src/error/types.rs`)

Single `Error` enum with `#[derive(thiserror::Error)]`:
`IoSimple`, `Io`, `Json`, `Generic`, `Utf8`, `Base64`, `InvalidRegex`, `NotFound`, `InvalidArgument`, `VectorDb`, `Embedding`, `Config`, `Configuration`, `ConfigMissing`, `ConfigInvalid`, `Authentication`, `Network`, `Database`, `Internal`, `Cache`, `Infrastructure`, `Vcs`, `RepositoryNotFound`, `BranchNotFound`, `ObservationStorage`, `ObservationNotFound`, `DuplicateObservation`, `Browse`, `Highlight`

Each variant has constructor methods: `Error::vcs("msg")`, `Error::database("msg")`, etc.

## Context Lifecycle (ADR-034..037, planned)

- Workflow FSM (ADR-034): Explicit state transitions for sessions
- Context Scout (ADR-035): Automatic project context discovery and caching
- Policy gates (ADR-036): Validate transitions and actions
- Orchestrator (ADR-037): Coordinate execution with compensation

## Integrated Context (ADR-041..046, planned)

- Knowledge graph for code relationships (ADR-042)
- Hybrid search: semantic + BM25 (ADR-043)
- Context versioning and freshness signals (ADR-045)

## Sources

- `crates/mcb-domain/src/entities/mod.rs`, `crates/mcb-domain/src/ports/mod.rs`
- `crates/mcb-domain/src/error/types.rs`, `crates/mcb-domain/src/lib.rs`

## Update Notes

- 2026-02-15: Full harvest rewrite — complete entity catalog, port traits, error model from source.
- 2026-02-12: Pointed to docs/modules/domain.md as source of truth.
