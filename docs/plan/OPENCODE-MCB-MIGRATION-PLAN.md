<!-- markdownlint-disable MD013 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# OpenCode → MCB Migration Plan

**Date**: 2026-02-08
**Version**: 1.0
**Scope**: Complete mapping of OpenCode components to MCB replacements

---

## Executive Summary

This document maps every OpenCode hook, skill, command, and agent to its MCB replacement across three phases (v0.2.0, v0.3.0, v0.4.0). Each mapping includes:

-   Current file location
-   Current implementation
-   MCB replacement
-   Migration action

---

## Current OpenCode Inventory

| Component Type | Count | Location |
|----------------|-------|----------|
| Hooks | 13 | `~/.config/opencode/hooks/*.sh` |
| Skills | 16 | `~/.config/opencode/skills/*/SKILL.md` |
| Commands | 44 | `~/.config/opencode/command/*.md` |
| Agents | 8 | Defined in `AGENTS.md` |
| Libraries | 10+ | `~/.config/opencode/lib/*.sh` |

---

## Phase 1: MCB v0.2.0 Integration (NOW)

### Available MCB Capabilities

| MCB Tool | Function | Status |
|----------|----------|--------|
| `mcp_mcb_index` | Index codebase | ✅ Working |
| `mcp_mcb_search` | Semantic search | ✅ Working |
| `mcp_mcb_session` | Session lifecycle | ✅ Working |
| `mcp_mcb_agent` | Agent activity logging | ✅ Working |
| `mcp_mcb_vcs` | Git operations | ⚠️ Partial |
| `mcp_mcb_validate` | Code validation | ✅ Working |
| `mcp_mcb_memory` | Observations | ❌ Blocked (GAP-H2) |
| `mcp_mcb_project` | Project workflow | ❌ Blocked (GAP-H1) |

---

### 1.1 Agent Replacements

#### Explore Agent → MCP_mcb_search

**Current Implementation**:

```
File: ~/.config/opencode/AGENTS.md (line 16)

6. **Explore** (Explorer): Codebase exploration and pattern analysis
   using `grep`, `glob`, and `ast-grep`.

Usage in delegation:
  task(subagent_type="explore", prompt="Find auth patterns...")
```

**MCB Replacement**:

```typescript
// Replace explore agent spawn with direct MCB call
mcp_mcb_search(
  query="authentication patterns",
  resource="code",
  collection="opencode",
  limit=10
)
```

**Migration Actions**:

| # | File | Action |
|---|------|--------|
| 1.1.1 | `~/.config/opencode/AGENTS.md` | Add MCB preference: "For semantic code queries, prefer `mcp_mcb_search` over spawning explore agent" |
| 1.1.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add section: "Replacing Explore Agent" with usage examples |
| 1.1.3 | `~/.config/opencode/oh-my-opencode.json` | Add `tool_preferences.code_search: ["mcp_mcb_search", "ast_grep_search", "grep"]` |

**When to KEEP explore agent**:

-   AST structural patterns (use ast-grep)
-   Exact regex matches (use grep)
-   Multi-file cross-reference (use LSP)

---

### 1.2 Session Tracking

#### oc-session-tracker Skill → MCP_mcb_session + MCP_mcb_agent

**Current Implementation**:

```
File: ~/.config/opencode/skills/oc-session-tracker/SKILL.md

Session Start (lines 12-23):
  memory(mode="add",
    content=`Session started: ${sessionId}
  Project: ${project}
  Directory: ${workDir}
  Branch: ${gitBranch}`,
    type="context",
    tags="session,start,${project}")

Session End (lines 54-66):
  memory(mode="add",
    content=`Session ended: ${sessionId}
  Duration: ${duration}
  Tasks completed: ${completedCount}/${totalCount}`,
    type="context",
    tags="session,end,${project}")
```

**MCB Replacement**:

```typescript
// Session Start
mcp_mcb_session(action="start", data={
  project_id: "opencode",
  context: {
    directory: workDir,
    branch: gitBranch,
    initial_message: userFirstMessage
  }
})

// Agent Activity
mcp_mcb_agent(action="log", data={
  session_id: sessionId,
  agent: "sisyphus",
  action: "delegated",
  context: "Sent to explore for auth patterns"
})

// Session End
mcp_mcb_session(action="end", data={
  session_id: sessionId,
  summary: {
    tasks_completed: completedCount,
    duration: duration,
    next_steps: pendingTasks
  }
})
```

**Migration Actions**:

| # | File | Action |
|---|------|--------|
| 1.2.1 | `~/.config/opencode/skills/oc-session-tracker/SKILL.md` | Add deprecation notice + MCB migration guide |
| 1.2.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Session Tracking" section |
| 1.2.3 | `~/.config/opencode/command/oc-welcome.md` | Replace `memory(mode="add")` with `mcp_mcb_session(action="start")` |

---

### 1.3 Codebase Indexing

#### /oc-init → Add MCB Index

**Current Implementation**:

```
File: ~/.config/opencode/command/oc-init.md

Creates:
-   .planning/ directory structure
-   ROADMAP.md, STATE.md, REQUIREMENTS.md
-   Beads initialization
-   NO semantic index created
```

**MCB Enhancement**:

```typescript
// After project initialization
mcp_mcb_index(
  action="start",
  collection="${project_name}",
  path="${project_path}",
  extensions=[".md", ".sh", ".ts", ".rs", ".py", ".json"]
)

// Store project registration
mcp_mcb_session(action="start", data={
  project_id: project_name,
  context: "Project initialized via /oc-init"
})
```

**Migration Actions**:

| # | File | Action |
|---|------|--------|
| 1.3.1 | `~/.config/opencode/command/oc-init.md` | Add `<mcb_index>` section after beads init |
| 1.3.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Project Indexing" section |

---

### 1.4 Memory Skill Enhancement

#### oc-memory Skill → MCP_mcb_memory (After GAP-H2 Fix)

**Current Implementation**:

```
File: ~/.config/opencode/skills/oc-memory/SKILL.md

Core Operations (lines 12-16):
| Capability | Memory Operation | When to Use |
|------------|------------------|-------------|
| **Recall** | `memory search` | Before any task |
| **Learn** | `memory add` | After completing tasks |
| **Profile** | `memory profile` | User preferences |

Usage (lines 34-44):
  memory(mode="search", query="[task keywords]")
  memory(mode="add", content="[learning]", type="[type]", tags="[tags]")
```

**MCB Replacement** (after GAP-H2 fix):

```typescript
// Search (same as before, better context)
mcp_mcb_memory(
  action="search",
  resource="observation",
  query="authentication patterns",
  project_id="opencode"  // NEW: project-scoped
)

// Store (project-linked)
mcp_mcb_memory(
  action="store",
  resource="observation",
  data={
    project_id: "opencode",
    content: "User prefers conventional commits with emoji",
    observation_type: "preference",
    tags: ["git", "commit", "style"]
  }
)
```

**Migration Actions** (Blocked until GAP-H2):

| # | File | Action | Blocked By |
|---|------|--------|------------|
| 1.4.1 | `~/.config/opencode/skills/oc-memory/SKILL.md` | Add MCB section with project-scoped examples | mcb-ibnx |
| 1.4.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Memory Integration" section | mcb-ibnx |

---

## Phase 2: MCB v0.3.0 Integration (After Release)

### New MCB Capabilities in v0.3.0

| Feature | ADR | MCB Tool |
|---------|-----|----------|
| Workflow FSM | ADR-034 | `mcp_mcb_workflow` |
| Context Scout | ADR-035 | Integrated in search |
| Policy Engine | ADR-036 | `mcp_mcb_workflow` policies |
| Persistent Memory | ADR-009 | `mcp_mcb_memory` (enhanced) |
| Git-Aware Index | ADR-008 | `mcp_mcb_vcs` (enhanced) |

---

### 2.1 Hook Replacements

#### oc-state-machine.sh → MCP_mcb_workflow

**Current Implementation**:

```
File: ~/.config/opencode/hooks/oc-state-machine.sh

State Management (lines 9-24):
  _default_session=$(cfg '.session.states[0]')  # NEW, RESUMED, ACTIVE, STALE
  _default_project=$(cfg '.project.states[0]')  # INIT, PLANNING, EXECUTING...

  _sess=$(state '.session.state')
  update_state '.session.state' "\"$_sess\""

Transitions (lines 25-50):
  Case-based state machine with manual transitions
```

**MCB Replacement**:

```typescript
// Replace shell-based FSM with MCB Workflow
mcp_mcb_workflow(
  action="transition",
  data={
    project_id: "opencode",
    from_state: "Active",
    to_state: "Paused",
    reason: "Awaiting user input",
    policies: ["freshness"]  // Validates context is fresh
  }
)

// Query current state
mcp_mcb_workflow(
  action="get_state",
  data={
    project_id: "opencode"
  }
)
// Returns: { state: "Active", since: "2026-02-08T15:00:00Z", policies_passed: true }
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 2.1.1 | `~/.config/opencode/hooks/oc-state-machine.sh` | Add MCB workflow integration | v0.3.0 release |
| 2.1.2 | `~/.config/opencode/lib/oc-state.sh` | Add MCB state sync functions | v0.3.0 release |
| 2.1.3 | `~/.config/opencode/oc-workflow.jsonc` | Map states to MCB workflow | v0.3.0 release |

---

#### oc-workflow-orchestration.sh → MCP_mcb_workflow

**Current Implementation**:

```
File: ~/.config/opencode/hooks/oc-workflow-orchestration.sh

Phases (lines 18-37):
  case "$_phase" in
    pre-commit)   oc_orchestrate_pre_commit "$_commit_msg" "$_auto_fix" ;;
    post-commit)  oc_orchestrate_post_commit ;;
    pre-delegation) oc_orchestrate_pre_delegation "$_task_type" ;;
    post-completion) oc_orchestrate_post_completion "$_issue_id" ;;
    full-validation) oc_orchestrate_full_validation "$_commit_msg" "error" ;;
  esac
```

**MCB Replacement**:

```typescript
// Pre-commit validation via MCB
mcp_mcb_workflow(
  action="gate_check",
  data={
    gate: "pre-commit",
    project_id: "opencode",
    policies: ["tests_pass", "lsp_clean", "no_debug_code"],
    context: { commit_msg: commitMsg }
  }
)
// Returns: { allowed: true } or { allowed: false, violations: [...] }

// Post-completion with context capture
mcp_mcb_workflow(
  action="complete",
  data={
    project_id: "opencode",
    task_id: issueId,
    capture: ["files_changed", "patterns_used", "decisions_made"]
  }
)
// Automatically stores to MCB memory
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 2.1.4 | `~/.config/opencode/hooks/oc-workflow-orchestration.sh` | Replace orchestration with MCB workflow gates | v0.3.0 release |
| 2.1.5 | `~/.config/opencode/lib/oc-workflow-orchestrator.sh` | Add MCB workflow client | v0.3.0 release |

---

### 2.2 Command Enhancements

#### /oc-welcome → MCB Session Integration

**Current Implementation**:

```
File: ~/.config/opencode/command/oc-welcome.md

Session Detection (lines 52-79):
  detectSessionState() {
    const lastActivity = await memory(mode="search", query="session start");
    const todos = await todoread();
    // Manual state detection logic
  }

Loads Skills (lines 12-18):
  load_skills:
    -   oc-workflow-integration
    -   oc-task-management
    -   oc-memory
    -   oc-session-tracker
```

**MCB Enhancement**:

```typescript
// Replace manual detection with MCB session
const session = await mcp_mcb_session(action="get_or_create", data={
  project_id: detectProject(),
  context: {
    git_branch: getGitBranch(),
    uncommitted: getUncommittedSummary()
  }
});

// Session state comes from MCB
if (session.is_new) {
  // Full discovery
} else if (session.is_stale) {
  // Warn + re-discover
} else {
  // Resume with delta
}
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 2.2.1 | `~/.config/opencode/command/oc-welcome.md` | Replace session detection with MCB | v0.3.0 release |
| 2.2.2 | Remove `oc-session-tracker` from load_skills | Add `oc-mcb` instead | v0.3.0 release |

---

#### /oc-plan → MCB Memory Patterns

**Current Implementation**:

```
File: ~/.config/opencode/command/oc-plan.md

Research Phase:
  -   Spawns librarian agent for external docs
  -   Reads .planning/*.md files manually
  -   No pattern memory search
```

**MCB Enhancement**:

```typescript
// Before planning, search for similar past plans
const patterns = await mcp_mcb_memory(
  action="search",
  resource="observation",
  query=`planning ${phaseGoal}`,
  project_id="opencode",
  filter={ observation_type: "pattern" }
);

// After planning, store approach
await mcp_mcb_memory(
  action="store",
  resource="observation",
  data={
    project_id: "opencode",
    content: `Phase ${phase} planned with ${taskCount} tasks using ${approach}`,
    observation_type: "pattern",
    tags: ["planning", "phase", phase]
  }
);
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 2.2.3 | `~/.config/opencode/command/oc-plan.md` | Add MCB pattern search before planning | v0.3.0 + GAP-H2 fix |
| 2.2.4 | `~/.config/opencode/command/oc-plan.md` | Add MCB pattern storage after planning | v0.3.0 + GAP-H2 fix |

---

### 2.3 Skill Deprecations

#### oc-memory → DEPRECATED (Migrate to oc-mcb)

**Migration Actions**:

| # | File | Action |
|---|------|--------|
| 2.3.1 | `~/.config/opencode/skills/oc-memory/SKILL.md` | Add deprecation header with migration guide |
| 2.3.2 | All commands using `mcp_memory` | Replace with `mcp_mcb_memory` |

**Deprecation Notice to Add**:

```markdown
---
name: oc-memory
status: DEPRECATED
superseded_by: oc-mcb
migration_date: 2026-Q1
---

# ⚠️ DEPRECATED - Use oc-mcb Memory

This skill is deprecated. Migrate to MCB memory for project-scoped observations.

## Migration Guide

| Before (oc-memory) | After (oc-mcb) |
|--------------------|----------------|
| `memory(mode="add", content="...", tags="...")` | `mcp_mcb_memory(action="store", resource="observation", data={project_id:"...", content:"...", tags:[...]})` |
| `memory(mode="search", query="...")` | `mcp_mcb_memory(action="search", resource="observation", query="...", project_id:"...")` |
```

---

#### oc-session-tracker → DEPRECATED (Migrate to oc-mcb)

**Migration Actions**:

| # | File | Action |
|---|------|--------|
| 2.3.3 | `~/.config/opencode/skills/oc-session-tracker/SKILL.md` | Add deprecation header |
| 2.3.4 | Commands loading this skill | Replace with `oc-mcb` |

---

## Phase 3: MCB v0.4.0 Integration (After Release)

### New MCB Capabilities in v0.4.0

| Feature | ADR | MCB Tool |
|---------|-----|----------|
| Knowledge Graph | ADR-042 | `mcp_mcb_search(resource="graph")` |
| Hybrid Search | ADR-043 | RRF fusion auto-enabled |
| Time-Travel | ADR-045 | `mcp_mcb_search(..., snapshot="v1.0")` |
| Context Search | ADR-041 | `mcp_mcb_search(resource="context")` |

---

### 3.1 Unified Context Search

#### Explore + Librarian → MCP_mcb_search(context)

**Current Implementation**:

```
File: ~/.config/opencode/AGENTS.md

Current flow for context gathering:
1. task(subagent_type="explore", prompt="Find X in code")
2. task(subagent_type="librarian", prompt="Find X in docs")
3. memory(mode="search", query="X decisions")
4. Manually combine results

Cost: 3 agent spawns + manual synthesis
Time: 30-60 seconds
```

**MCB Replacement**:

```typescript
// Single call replaces all three
const context = await mcp_mcb_search(
  resource="context",  // Unified search
  query="authentication implementation",
  project_id="opencode",
  include=["code", "memory", "session", "vcs"],
  freshness_max_age=7  // Only patterns < 7 days old
);

// Returns:
{
  code_matches: [...],      // From indexed codebase
  memory_matches: [...],    // From observations
  session_context: [...],   // From recent sessions
  vcs_context: {...},       // From git state
  freshness: "fresh"        // Or "stale" with warning
}

Cost: 0 agent spawns
Time: 5-10 seconds
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 3.1.1 | `~/.config/opencode/AGENTS.md` | Update delegation table: context queries → MCB | v0.4.0 release |
| 3.1.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Unified Context Search" section | v0.4.0 release |

---

### 3.2 Knowledge Graph

#### oc-cartography → MCP_mcb_search(graph)

**Current Implementation**:

```
File: ~/.config/opencode/skills/oc-cartography/SKILL.md

Operations:
  python3 cartographer.py init    # Creates .slim/cartography.json
  python3 cartographer.py changes # Detect changed dirs
  python3 cartographer.py update  # Update codemaps

Output: codemap.md files per directory (static snapshots)
```

**MCB Replacement**:

```typescript
// Dynamic code structure query
const structure = await mcp_mcb_search(
  resource="graph",
  query="module_structure",
  data={
    collection: "opencode",
    root: "lib/",
    depth: 2
  }
);

// Impact analysis before refactoring
const impact = await mcp_mcb_search(
  resource="graph",
  query="impact_analysis",
  data={
    file: "lib/oc-core.sh",
    depth: 3
  }
);
// Returns: all files/functions affected by changes

// Dependency graph
const deps = await mcp_mcb_search(
  resource="graph",
  query="dependencies",
  data={
    target: "hooks/oc-state-machine.sh"
  }
);
// Returns: what this file imports/uses
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 3.2.1 | `~/.config/opencode/skills/oc-cartography/SKILL.md` | Add deprecation notice | v0.4.0 release |
| 3.2.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Knowledge Graph" section | v0.4.0 release |
| 3.2.3 | `~/.config/opencode/command/oc-refactor.md` | Add MCB impact analysis | v0.4.0 release |
| 3.2.4 | `~/.config/opencode/command/oc-analyze-patterns.md` | Replace cartography with MCB graph | v0.4.0 release |

---

### 3.3 Time-Travel Debugging

#### /oc-debug → MCB Snapshot Queries

**Current Implementation**:

```
File: ~/.config/opencode/command/oc-debug.md

Current debugging:
-   Manual git checkout to previous versions
-   Read code, compare manually
-   No semantic comparison
```

**MCB Enhancement**:

```typescript
// Find when behavior changed
const beforeRelease = await mcp_mcb_search(
  query="validate_token implementation",
  collection="opencode",
  snapshot="v1.0.0"  // Search at this tag
);

const current = await mcp_mcb_search(
  query="validate_token implementation",
  collection="opencode"
);

// Compare semantically
const diff = await mcp_mcb_vcs(
  action="semantic_diff",
  data={
    collection: "opencode",
    base_snapshot: "v1.0.0",
    head_snapshot: "current",
    query: "authentication"
  }
);
// Returns: what changed semantically in auth code
```

**Migration Actions**:

| # | File | Action | Depends On |
|---|------|--------|------------|
| 3.3.1 | `~/.config/opencode/command/oc-debug.md` | Add MCB time-travel section | v0.4.0 release |
| 3.3.2 | `~/.config/opencode/skills/oc-mcb/SKILL.md` | Add "Time-Travel Debugging" section | v0.4.0 release |

---

## Complete File Change Summary

### Phase 1 (v0.2.0) - 8 Files

| File | Change Type | Priority |
|------|-------------|----------|
| `AGENTS.md` | MODIFY - Add MCB preferences | P1 |
| `skills/oc-mcb/SKILL.md` | MODIFY - Add sections | P1 |
| `skills/oc-session-tracker/SKILL.md` | MODIFY - Add deprecation | P2 |
| `command/oc-welcome.md` | MODIFY - Add MCB session | P1 |
| `command/oc-init.md` | MODIFY - Add MCB index | P2 |
| `oh-my-opencode.json` | MODIFY - Add tool preferences | P2 |
| `skills/oc-memory/SKILL.md` | MODIFY - Add MCB section | P2 |

### Phase 2 (v0.3.0) - 12 Files

| File | Change Type | Priority |
|------|-------------|----------|
| `hooks/oc-state-machine.sh` | MODIFY - Add MCB workflow | P1 |
| `hooks/oc-workflow-orchestration.sh` | MODIFY - Add MCB gates | P1 |
| `lib/oc-state.sh` | MODIFY - Add MCB sync | P1 |
| `lib/oc-workflow-orchestrator.sh` | MODIFY - Add MCB client | P1 |
| `oc-workflow.jsonc` | MODIFY - Map to MCB | P2 |
| `command/oc-welcome.md` | MODIFY - MCB session | P1 |
| `command/oc-plan.md` | MODIFY - MCB patterns | P2 |
| `skills/oc-memory/SKILL.md` | DEPRECATE | P2 |
| `skills/oc-session-tracker/SKILL.md` | DEPRECATE | P2 |

### Phase 3 (v0.4.0) - 6 Files

| File | Change Type | Priority |
|------|-------------|----------|
| `AGENTS.md` | MODIFY - Update delegation | P1 |
| `skills/oc-cartography/SKILL.md` | DEPRECATE | P2 |
| `command/oc-refactor.md` | MODIFY - Add impact analysis | P1 |
| `command/oc-analyze-patterns.md` | MODIFY - MCB graph | P2 |
| `command/oc-debug.md` | MODIFY - Time-travel | P2 |
| `skills/oc-mcb/SKILL.md` | MODIFY - Add all new sections | P1 |

---

## Deprecation Schedule

| Component | v0.2.0 | v0.3.0 | v0.4.0 | v1.0.0 |
|-----------|--------|--------|--------|--------|
| explore agent (semantic) | Deprecated | Warn | Removed | - |
| explore agent (structural) | Keep | Keep | Keep | Keep |
| oc-memory skill | Keep | Deprecated | Warn | Removed |
| oc-session-tracker skill | Deprecated | Warn | Removed | - |
| oc-cartography skill | Keep | Keep | Deprecated | Removed |
| librarian (internal) | Keep | Deprecated | Warn | Removed |
| librarian (external) | Keep | Keep | Keep | Keep |
| Shell hooks (state) | Keep | Deprecated | Warn | Removed |

---

## Beads Issues Summary

### MCB Repository Issues (10)

| Issue | Type | Priority | Gap |
|-------|------|----------|-----|
| mcb-ibnx | bug | P0 | Memory query fails |
| mcb-e2uy | feature | P0 | Project not implemented |
| mcb-v9o3 | feature | P0 | CodeGraph entity |
| mcb-onib | feature | P0 | TantivyBM25 |
| mcb-ftvv | feature | P0 | TreeSitterExtractor |
| mcb-llfp | feature | P0 | ContextSnapshot |
| mcb-4u57 | feature | P0 | WorkflowEngine port |
| mcb-vist | feature | P1 | Context search |
| mcb-6hsv | feature | P1 | WorkflowState 12-state |
| mcb-pjuc | bug | P1 | VCS fix |

### OpenCode Repository Issues (3)

| Issue | Type | Priority | Phase |
|-------|------|----------|-------|
| opencode-o5t8 | feature | P1 | Phase 1 - Replace explore |
| opencode-rn79 | feature | P1 | Phase 1 - MCB session |
| opencode-5mcn | feature | P2 | Phase 1 - MCB index |

---

## Success Metrics

| Metric | Before MCB | After v0.2.0 | After v0.3.0 | After v0.4.0 |
|--------|------------|--------------|--------------|--------------|
| Agent spawns/task | 2-4 | 1-2 | 0-1 | 0-1 |
| Context gather time | 30-60s | 10-20s | 5-10s | 3-5s |
| Memory persistence | Session | Session | Permanent | Permanent |
| Cross-session context | None | Basic | Full | Full + temporal |
| Code understanding | Manual | Semantic | Semantic | Graph + semantic |

---

## Cross-References

-   **MCB Gaps Report**: `/home/marlonsc/mcb/docs/plan/MCB-COMPREHENSIVE-GAPS.md`
-   **MCB Roadmap**: `/home/marlonsc/mcb/docs/developer/ROADMAP.md`
-   **OpenCode AGENTS.md**: `/home/marlonsc/.config/opencode/AGENTS.md`
-   **OpenCode Skills**: `/home/marlonsc/.config/opencode/skills/`
-   **OpenCode Commands**: `/home/marlonsc/.config/opencode/command/`
-   **OpenCode Hooks**: `/home/marlonsc/.config/opencode/hooks/`
