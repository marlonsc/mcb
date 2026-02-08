# MCB-OpenCode Integration Plan

**Date**: 2026-02-08  
**Goal**: Replace heavy agent work with MCB project-memory-context pattern

---

## Current State (Heavy Agent Work)

```
┌──────────────────────────────────────────────────────────────┐
│  Current OpenCode Workflow                                   │
├──────────────────────────────────────────────────────────────┤
│  explore agent ──────> grep/ast-grep ──────> results         │
│  librarian agent ────> context7/web ──────> docs             │
│  oc-memory skill ────> mcp_memory ────────> observations     │
│  oc-session-tracker ─> manual tracking ───> session logs     │
│  beads ──────────────> bd CLI ────────────> issues           │
└──────────────────────────────────────────────────────────────┘

Problems:
- Each agent spawns, reads files, searches - expensive
- No unified context across agents
- Memory/observations not linked to project
- Session context lost between invocations
```

---

## Target State (MCB-Powered)

```
┌──────────────────────────────────────────────────────────────┐
│  MCB-Integrated Workflow                                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Project: "opencode"                                         │
│  ├── Collection: indexed codebase (semantic search)         │
│  ├── Memory: project-scoped observations                    │
│  ├── Sessions: agent sessions with full context             │
│  └── VCS: git-aware change tracking                         │
│                                                              │
│  Agent Request ───┐                                          │
│                   ▼                                          │
│  ┌─────────────────────────────────────────┐                │
│  │  MCB Unified Search                     │                │
│  │  - Code: semantic search collection     │                │
│  │  - Memory: project observations         │                │
│  │  - Context: session + git state         │                │
│  └─────────────────────────────────────────┘                │
│                   │                                          │
│                   ▼                                          │
│  Single response with relevant code + memory + context       │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Benefits:
- One call replaces explore + librarian agents
- Unified context across all searches
- Memory persists and links to project
- Session context preserved
```

---

## Integration Phases

### Phase A: Basic Integration (MCB v0.2.0 - NOW)

**Available Now**:

-   `mcp_mcb_search(resource="code")` - Semantic code search
-   `mcp_mcb_index` - Index codebase
-   `mcp_mcb_session` - Basic session lifecycle
-   `mcp_mcb_vcs` - Git repository awareness

**Replace**:

| Current | MCB Replacement | Savings |
|---------|-----------------|---------|
| `explore` agent for code patterns | `mcp_mcb_search(resource="code")` | -1 agent spawn |
| `grep` for semantic queries | `mcp_mcb_search` | Better relevance |
| Manual session tracking | `mcp_mcb_session` | Automatic |

**OpenCode Changes**:

```typescript
// Before: Spawn explore agent
task(subagent_type="explore", prompt="Find auth patterns...")

// After: Direct MCB search
mcp_mcb_search(query="authentication patterns", collection="opencode", limit=10)
```

---

### Phase B: Memory Integration (After GAP-2 Fix)

**Requires**: mcb-ibnx (Memory query fix)

**Replace**:

| Current | MCB Replacement | Savings |
|---------|-----------------|---------|
| `mcp_memory` skill | `mcp_mcb_memory` | Unified storage |
| `oc-memory` observations | Project-linked observations | Context aware |
| Pattern storage | MCB memory with embeddings | Semantic recall |

**OpenCode Changes**:

```typescript
// Before: Separate memory tool
mcp_memory(mode="add", content="User prefers X", tags="preference")

// After: Project-linked memory
mcp_mcb_memory(action="store", resource="observation", data={
  project_id: "opencode",
  content: "User prefers X",
  observation_type: "preference"
})
```

---

### Phase C: Project Integration (After GAP-1 Fix)

**Requires**: mcb-e2uy (Project workflow implementation)

**Replace**:

| Current | MCB Replacement | Savings |
|---------|-----------------|---------|
| `.planning/` directory | MCB project phases | Unified tracking |
| Beads issues (partial) | MCB project issues | Linked to code |
| Manual context gathering | Project-scoped queries | Automatic |

**OpenCode Changes**:

```typescript
// Before: Read .planning files
read(".planning/ROADMAP.md")
read(".planning/STATE.md")

// After: MCB project state
mcp_mcb_project(action="get", resource="phase", project_id="opencode")
```

---

### Phase D: Unified Context (After GAP-4 Fix)

**Requires**: mcb-vist (Context search handler)

**Replace**:

| Current | MCB Replacement | Savings |
|---------|-----------------|---------|
| explore + librarian + memory | Single context search | -2 agents |
| Multi-step context gathering | One unified query | Faster |
| Manual context assembly | Automatic fusion | Better quality |

**OpenCode Changes**:

```typescript
// Before: Multiple agent spawns
task(subagent_type="explore", prompt="Find X in code")
task(subagent_type="librarian", prompt="Find X in docs")
mcp_memory(mode="search", query="X")

// After: Unified context search
mcp_mcb_search(resource="context", query="X", project_id="opencode")
// Returns: code matches + memory observations + session context
```

---

## Skill Updates Required

### 1. Update oc-mcb Skill

Add project-aware wrappers:

```markdown
## Project-Aware Usage

# Initialize project (once)
mcp_mcb_project(action="create", project_id="opencode", data={
  path: "~/.config/opencode",
  collection: "opencode"
})

# All subsequent calls are project-scoped
mcp_mcb_search(query="...", project_id="opencode")
mcp_mcb_memory(project_id="opencode", ...)
```

### 2. Deprecate Redundant Skills (After Full Integration)

| Skill | Status | Replacement |
|-------|--------|-------------|
| `oc-memory` | Deprecate | `mcp_mcb_memory` |
| `oc-session-tracker` | Deprecate | `mcp_mcb_session` |
| `oc-cartography` | Keep | Complements MCB (structure vs semantic) |

### 3. Update Agent Delegation

```typescript
// In AGENTS.md, update delegation table:
| Domain | Current | After MCB |
|--------|---------|-----------|
| Code patterns | explore agent | mcp_mcb_search |
| Memory/observations | mcp_memory | mcp_mcb_memory |
| Session context | manual | mcp_mcb_session |
| Project state | .planning files | mcp_mcb_project |
```

---

## Validation Criteria

### Phase A (Now)

-   [ ] `mcp_mcb_search` returns relevant code for natural language
-   [ ] Relevance scores > 0.3 (currently ~0.05)
-   [ ] Index includes all file types (.md, .sh, .JSON, etc.)

### Phase B (After GAP-2)

-   [ ] `mcp_mcb_memory(action="store")` succeeds
-   [ ] `mcp_mcb_memory(action="list")` returns stored observations
-   [ ] Memory search returns semantically similar observations

### Phase C (After GAP-1)

-   [ ] `mcp_mcb_project(action="create")` succeeds
-   [ ] Project links to collection and memory
-   [ ] Phase tracking works via MCB

### Phase D (After GAP-4)

-   [ ] `mcp_mcb_search(resource="context")` returns unified results
-   [ ] Single query replaces explore + librarian + memory
-   [ ] Agent spawn count reduced by 50%+

---

## Metrics to Track

| Metric | Before MCB | Target | How to Measure |
|--------|------------|--------|----------------|
| Agent spawns per task | 2-4 | 0-1 | Count task() calls |
| Context gathering time | 30-60s | 5-10s | Time to first relevant Result |
| Memory persistence | Session-only | Permanent | Check MCB memory.db |
| Cross-session context | None | Full | MCB project state |

---

## Blockers Summary

| Blocker | Issue | Priority | Status |
|---------|-------|----------|--------|
| Memory query fails | mcb-ibnx | P0 | Open |
| Project not implemented | mcb-e2uy | P0 | Open |
| Context search missing | mcb-vist | P1 | Open |
| Low relevance scores | - | P2 | Investigate embeddings |

---

## Next Actions

1.  **Immediate**: Use Phase A capabilities (code search only)
2.  **Agent work**: Fix mcb-ibnx (memory query) - highest impact
3.  **Agent work**: Fix mcb-e2uy (project workflow) - enables full pattern
4.  **Validation**: Re-run integration tests after each fix
