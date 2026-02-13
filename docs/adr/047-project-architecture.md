---
adr: 47
title: Project Architecture - Central Hub and Multi-Dimensional Coordination
status: PROPOSED
created: 2026-02-08
updated: 2026-02-08
related: [14, 34, 41]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 047: Project Architecture - Central Hub and Multi-Dimensional Coordination

## Status

**Proposed** - 2026-02-08

## Context

MCB currently lacks a centralized "Project" entity that coordinates various dimensions of the system. While it provides semantic search and workflow management, these are often disconnected. Gaps identified in the comprehensive gap analysis (GAP-H1, GAP-E2) highlight the need for a unified project architecture that serves as a central hub for:

1. **Project-Repository Link**: 1:1 mapping between projects and git repositories.
2. **Multi-Collection Support**: Handling multiple vector collections per worktree.
3. **Multi-Session Management**: Tracking user, agent, and worktree context across sessions.
4. **Multi-User & Multi-Agent Coordination**: Supporting parallel agent sessions and operator roles.
5. **Worktree Support**: Ensuring each git worktree has an isolated semantic index.

## Decision

We will implement a unified `Project` architecture in `mcb-domain` and `mcb-server` to serve as the central hub for all operations.

### 1. Project Entity Model

The `Project` entity will encapsulate the link between version control, semantic indexing, and active workflows.

```rust
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub root_path: PathBuf,
    pub repository_url: String,
    pub worktrees: Vec<WorktreeInfo>,
    pub collections: Vec<CollectionId>,
    pub metadata: ProjectMetadata,
}

pub struct WorktreeInfo {
    pub id: WorktreeId,
    pub path: PathBuf,
    pub branch: String,
    pub head_commit: String,
    pub index_id: IndexId,
}
```

### 2. Project-Scoped Handlers (GAP-H1)

The currently stubbed project handlers in `mcb-server/src/handlers/project.rs` will be implemented to support CRUD operations and link projects to active services.

### 3. Worktree-Isolated Indexing

Each worktree within a project will maintain its own isolated semantic index (Milvus collection or Tantivy index), preventing context leakage between branches.

### 4. Multi-Agent Coordination

Projects will track active agent sessions, allowing parallel agents to work on the same project while maintaining shared context and individual session state.

## Consequences

### Positive Consequences

- **Contextual Integrity**: Worktree isolation ensures that search results are always relevant to the current branch.
- **Session Continuity**: Projects provide a stable anchor for long-running multi-agent workflows.
- **Scalability**: Proper collection management allows MCB to scale to large monorepos with multiple worktrees.

### Negative Consequences

- **Increased Complexity**: Managing multiple worktrees and collections adds overhead to the indexing service.
- **Storage Requirements**: Isolated indexes per worktree will increase disk/memory usage.

## Alternatives Considered

### Alternative 1: Single Index for all Worktrees

- **Description**: Use a single vector collection for the entire repository, using metadata filters for branches.
- **Pros**: Simpler storage model, less overhead during initial indexing.
- **Cons**: Filter performance degrades with many branches; high risk of "context pollution" if metadata is missing.
- **Rejection Reason**: Violates the requirement for strict worktree isolation and accurate semantic search.

## Implementation Notes

- **Domain Layer**: Add `Project` and `WorktreeInfo` entities to `mcb-domain`.
- **Infrastructure Layer**: Update `ProjectService` to handle git worktree discovery.
- **Server Layer**: Implement full CRUD in `mcb-server/src/handlers/project.rs`.

## References

- [MCB Comprehensive Gap Analysis](../plans/archive/MCB-COMPREHENSIVE-GAPS.md)
- [ADR-014: Multi-Domain Architecture Strategy](./014-multi-domain-architecture.md)
- [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md)
