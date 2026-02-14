<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# MCB Gaps Report - Integration Blockers

**Date**: 2026-02-08
**Reporter**: Sisyphus Agent
**Context**: OpenCode MCP Integration Validation

---

## Executive Summary

MCB v0.2.0 was validated for OpenCode integration. While basic search works, **critical gaps block the project-memory-context integration pattern**.

---

## Gap Analysis

### GAP-1: Project Workflow Not Implemented (CRITICAL)

**Tool**: `mcp_mcb_project`
**Status**: Returns "Project workflow not yet implemented"
**Impact**: Cannot link collections to projects, cannot use project-scoped memory

**Expected Behavior**:

```rust
// Create project linked to collection
mcb_project(action="create", resource="phase", project_id="opencode", data={...})
```

**Actual Behavior**:

```
Project workflow not yet implemented
```

**Required Fix**:

-   Implement ProjectService in mcb-application
-   Wire project MCP handlers in mcb-server
-   Link project_id to collections and observations

**Blocked Features**:

-   Project-scoped semantic search
-   Project-linked memory/observations
-   Phase tracking via MCB

---

### GAP-2: Memory List/Query Fails (CRITICAL)

**Tool**: `mcp_mcb_memory`
**Status**: SQL query_all error on list operation
**Impact**: Cannot retrieve stored observations

**Error**:

```
Failed to list memories: Observation storage error: SQL query_all failed
```

**Database Status**:

-   Tables exist: `observations`, `observations_fts`, etc.
-   Schema correct with FTS triggers
-   0 rows in observations table
-   projects table may be empty (foreign key constraint)

**Root Cause Hypothesis**:
The observations table has `project_id TEXT NOT NULL REFERENCES projects(id)`. If no projects exist, observations cannot be created. The memory store operation may silently fail or the list query joins on non-existent projects.

**Required Fix**:

1. Check if projects table has entries
2. Memory store should auto-create default project if none exists
3. Memory list should handle empty project gracefully
4. Add better error messages

---

### GAP-3: VCS Has 500+ Test Collections (LOW)

**Tool**: `mcp_mcb_vcs`
**Status**: Working but polluted with test data
**Impact**: Performance degradation, confusing output

**Observation**:

```
repositories: [
  "TestUpperCase-...",  // 300+ entries
  "my-project-hyphens-...",  // 200+ entries
  "persistent-test-...",  // 200+ entries
  "opencode",  // actual collection
  ...
]
```

**Required Fix**:

-   Add cleanup script for test collections
-   Consider test isolation (separate Milvus namespace)
-   Add `mcb cleanup --test-data` command

---

### GAP-4: Context Search Handler Missing (BLOCKED)

**Tool**: `mcp_mcb_search` with `resource="context"`
**Status**: SearchResource enum only has Code and Memory
**Impact**: Cannot search unified context (code + memory + session)

**Per v030-IMPLEMENTATION.md**:

-   SearchResource needs Context variant
-   SearchHandler needs ContextServiceInterface injection
-   This is blocking 15+ beads issues

---

## Working Features

| Feature | Status | Notes |
| --------- | -------- | ------- |
| Index codebase | ✅ | Works with Milvus |
| Search code | ✅ | Low relevance scores (~0.05) |
| Session list | ✅ | Returns empty (expected) |
| VCS list | ✅ | Works but has test pollution |
| Validate | ✅ | Not tested |

---

## Integration Pattern (Blocked)

The intended integration pattern uses **Project as Central Hub**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  PROJECT: "opencode" (Central Entity)                                   │
│  ├── repository_path: ~/.config/opencode                                │
│  ├── git_remote: github.com/user/opencode                               │
│  │                                                                      │
│  ├── COLLECTIONS (1:N per project)                                      │
│  │   ├── opencode-main (main branch index)                              │
│  │   ├── opencode-feature-auth (worktree index)                         │
│  │   └── opencode-hotfix-123 (worktree index)                           │
│  │                                                                      │
│  ├── MEMORY (project-scoped observations)                               │
│  │   ├── patterns: architectural decisions                              │
│  │   ├── preferences: user/team preferences                             │
│  │   └── errors: learned error patterns                                 │
│  │                                                                      │
│  ├── SESSIONS (multi-agent, multi-user)                                 │
│  │   ├── ses_abc (user: alice, agent: sisyphus, worktree: main)         │
│  │   ├── ses_def (user: bob, agent: explore, worktree: feature-auth)    │
│  │   └── ses_ghi (user: alice, agent: oracle, worktree: main)           │
│  │                                                                      │
│  ├── WORKTREES (parallel development)                                   │
│  │   ├── main → ~/.config/opencode                                      │
│  │   ├── feature-auth → ~/.config/opencode-feature-auth                 │
│  │   └── hotfix-123 → ~/.config/opencode-hotfix-123                     │
│  │                                                                      │
│  └── OPERATORS (who can work on project)                                │
│      ├── user: alice (owner)                                            │
│      ├── user: bob (contributor)                                        │
│      └── agent: ci-bot (automated)                                      │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Architecture Decisions**:

1. **Project = Repository**: 1:1 mapping to git repository
2. **Collection per Worktree**: Each git worktree has its own index
3. **Memory is Project-Scoped**: Observations belong to project, not session
4. **Sessions are Multi-Dimensional**: Track user + agent + worktree
5. **Operators Control Access**: Users and automated agents have roles

**Current State**: Only single collection works. Project entity, multi-worktree, multi-user, and multi-agent support are NOT implemented.

---

## Recommended Fix Priority

| Priority | Gap | Effort | Impact |
| ---------- | ----- | -------- | -------- |
| P0 | GAP-2 Memory fails | 2-4h | Unlocks observation storage |
| P0 | GAP-1 Project not implemented | 1-2d | Unlocks project-scoped context |
| P1 | GAP-4 Context search | 4-8h | Unlocks unified search |
| P2 | GAP-3 Test data cleanup | 2h | Cleanup only |

---

## Beads Issues to Create

```bash
cd ~/mcb
bd create --title "GAP-2: Memory list/query SQL error" --type bug --priority 0
bd create --title "GAP-1: Project workflow not implemented" --type feature --priority 0
bd create --title "GAP-4: Context search handler missing" --type feature --priority 1
bd create --title "GAP-3: Cleanup test collections from Milvus" --type task --priority 2
```

---

## Workarounds for OpenCode Integration

Until gaps are fixed, OpenCode can use MCB for:

1. **Code search only** - `mcp_mcb_search(resource="code", collection="opencode")`
2. **Manual session tracking** - Use `mcp_mcb_session` for basic lifecycle
3. **Skip project/memory** - Use existing `mcp_memory` skill instead

---

## Next Steps

1. Agent should implement GAP-2 fix (memory query)
2. Then implement GAP-1 (project workflow)
3. Then GAP-4 (context search)
4. Revalidate integration after each fix
