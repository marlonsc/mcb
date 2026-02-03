# Agent & Quality Domain Extension - Schema Additions

## Overview

**Adições incrementais** ao schema MCB existente para suportar agent tracking e quality enforcement. Segue os padrões de nomenclatura dos ADRs existentes (ADR-009, ADR-013).

## Design Principles

1.  **Extend, don't replace** - Usar infraestrutura existente
2.  **Naming per ADR-009** - Tools: `agent_`, `quality_`, `memory_`, `project_`
3.  **Tables per MCB pattern** - Entity names sem prefixos genéricos

---

## Part 1: Entity Extensions

### 1.1 ObservationType Enum Extension

**Existing** (mcb-domain/src/entities/memory.rs):

```rust
pub enum ObservationType {
    Code,
    Decision,
    Context,
    Error,
    Summary,
}
```

**Add**:

```rust
pub enum ObservationType {
    Code,
    Decision,
    Context,
    Error,
    Summary,
    Execution,  // NEW: test/lint/build executions
}
```

### 1.2 ExecutionMetadata (new struct)

Para Observations type=Execution:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub command: String,           // "make test", "cargo clippy"
    pub exit_code: i32,
    pub duration_ms: u64,
    pub success: bool,
    pub execution_type: ExecutionType,
    pub coverage: Option<f32>,
    pub files_affected: Vec<String>,
    pub output_summary: String,
    pub warnings_count: Option<u32>,
    pub errors_count: Option<u32>,
}

pub enum ExecutionType {
    Test,
    Lint,
    Build,
    CI,
}
```

### 1.3 ErrorMetadata Extension

**Add to Error observation metadata**:

```rust
pub struct ErrorMetadata {
    // ... existing fields ...
    pub fix_pattern: Option<String>,    // How this was fixed
    pub fix_verified: bool,             // Was the fix confirmed working
    pub error_hash: String,             // For matching similar errors
}
```

---

## Part 2: Agent Tables

FK to existing `session_summaries`.

### 2.1 agent_sessions

```sql
CREATE TABLE agent_sessions (
    id                  TEXT PRIMARY KEY,
    session_summary_id  TEXT NOT NULL REFERENCES session_summaries(id),
    agent_type          TEXT NOT NULL,       -- "sisyphus", "oracle", "explore"
    model               TEXT NOT NULL,       -- "claude-opus-4-5"
    parent_session_id   TEXT REFERENCES agent_sessions(id),
    started_at          INTEGER NOT NULL,
    ended_at            INTEGER,
    duration_ms         INTEGER,
    status              TEXT NOT NULL DEFAULT 'active',
    prompt_summary      TEXT,
    result_summary      TEXT,
    token_count         INTEGER,
    tool_calls_count    INTEGER DEFAULT 0,
    delegations_count   INTEGER DEFAULT 0
);

CREATE INDEX idx_agent_sessions_parent ON agent_sessions(parent_session_id);
CREATE INDEX idx_agent_sessions_type ON agent_sessions(agent_type);
CREATE INDEX idx_agent_sessions_started ON agent_sessions(started_at);
```

### 2.2 delegations

```sql
CREATE TABLE delegations (
    id                  TEXT PRIMARY KEY,
    parent_session_id   TEXT NOT NULL REFERENCES agent_sessions(id),
    child_session_id    TEXT NOT NULL REFERENCES agent_sessions(id),
    prompt              TEXT NOT NULL,
    prompt_embedding_id TEXT,
    result              TEXT,
    success             BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          INTEGER NOT NULL,
    completed_at        INTEGER,
    duration_ms         INTEGER
);

CREATE INDEX idx_delegations_parent ON delegations(parent_session_id);
CREATE INDEX idx_delegations_child ON delegations(child_session_id);
```

### 2.3 tool_calls

```sql
CREATE TABLE tool_calls (
    id                  TEXT PRIMARY KEY,
    session_id          TEXT NOT NULL REFERENCES agent_sessions(id),
    tool_name           TEXT NOT NULL,
    params_summary      TEXT,
    success             BOOLEAN NOT NULL,
    error_message       TEXT,
    duration_ms         INTEGER,
    created_at          INTEGER NOT NULL
);

CREATE INDEX idx_tool_calls_session ON tool_calls(session_id);
CREATE INDEX idx_tool_calls_tool ON tool_calls(tool_name);
```

### 2.4 checkpoints

```sql
CREATE TABLE checkpoints (
    id                  TEXT PRIMARY KEY,
    session_id          TEXT NOT NULL REFERENCES agent_sessions(id),
    checkpoint_type     TEXT NOT NULL,       -- "git", "file", "config"
    description         TEXT NOT NULL,
    snapshot_data       TEXT NOT NULL,       -- JSON
    created_at          INTEGER NOT NULL,
    restored_at         INTEGER,
    expired             BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_checkpoints_session ON checkpoints(session_id);
```

---

## Part 3: Project State Tables

Adições ao `ProjectSchema`.

### 3.1 phases

GSD phases:

```sql
CREATE TABLE phases (
    id                  TEXT PRIMARY KEY,
    project_id          TEXT NOT NULL REFERENCES projects(id),
    phase_number        TEXT NOT NULL,       -- "70", "80.1"
    name                TEXT NOT NULL,
    goal                TEXT,
    status              TEXT NOT NULL DEFAULT 'pending',
    progress            INTEGER DEFAULT 0,
    depends_on          TEXT,                -- JSON array
    started_at          INTEGER,
    completed_at        INTEGER,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    
    UNIQUE(project_id, phase_number)
);

CREATE INDEX idx_phases_project ON phases(project_id);
CREATE INDEX idx_phases_status ON phases(status);
```

### 3.2 issues

Beads issues:

```sql
CREATE TABLE issues (
    id                  TEXT PRIMARY KEY,    -- "mcb-7b2"
    project_id          TEXT NOT NULL REFERENCES projects(id),
    phase_id            TEXT REFERENCES phases(id),
    title               TEXT NOT NULL,
    description         TEXT,
    issue_type          TEXT NOT NULL DEFAULT 'task',
    priority            INTEGER NOT NULL DEFAULT 2,
    status              TEXT NOT NULL DEFAULT 'open',
    assignee            TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    closed_at           INTEGER,
    close_reason        TEXT,
    metadata            TEXT                 -- JSON
);

CREATE INDEX idx_issues_project ON issues(project_id);
CREATE INDEX idx_issues_phase ON issues(phase_id);
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
```

### 3.3 issue_dependencies

```sql
CREATE TABLE issue_dependencies (
    id                  TEXT PRIMARY KEY,
    issue_id            TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    depends_on_id       TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    dependency_type     TEXT NOT NULL DEFAULT 'blocks',
    created_at          INTEGER NOT NULL,
    
    UNIQUE(issue_id, depends_on_id, dependency_type)
);

CREATE INDEX idx_issue_deps_issue ON issue_dependencies(issue_id);
CREATE INDEX idx_issue_deps_depends ON issue_dependencies(depends_on_id);
```

### 3.4 decisions

```sql
CREATE TABLE decisions (
    id                  TEXT PRIMARY KEY,
    project_id          TEXT NOT NULL REFERENCES projects(id),
    phase_id            TEXT REFERENCES phases(id),
    session_id          TEXT REFERENCES agent_sessions(id),
    decision            TEXT NOT NULL,
    rationale           TEXT,
    outcome             TEXT,
    decided_at          INTEGER NOT NULL,
    created_at          INTEGER NOT NULL
);

CREATE INDEX idx_decisions_project ON decisions(project_id);
CREATE INDEX idx_decisions_phase ON decisions(phase_id);
```

---

## Part 4: Quality Gate Table

### 4.1 quality_gate_configs

```sql
CREATE TABLE quality_gate_configs (
    id                  TEXT PRIMARY KEY,
    project_id          TEXT NOT NULL REFERENCES projects(id),
    require_tests       BOOLEAN DEFAULT TRUE,
    require_lint        BOOLEAN DEFAULT TRUE,
    require_build       BOOLEAN DEFAULT FALSE,
    min_coverage        REAL,
    max_warnings        INTEGER,
    enabled             BOOLEAN DEFAULT TRUE,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    
    UNIQUE(project_id)
);
```

---

## Part 5: Views

### 5.1 ready_issues

```sql
CREATE VIEW ready_issues AS
WITH blocked AS (
    SELECT DISTINCT d.issue_id
    FROM issue_dependencies d
    JOIN issues blocker ON d.depends_on_id = blocker.id
    WHERE blocker.status NOT IN ('closed')
      AND d.dependency_type = 'blocks'
)
SELECT i.*
FROM issues i
WHERE i.status = 'open'
  AND i.id NOT IN (SELECT issue_id FROM blocked)
ORDER BY i.priority, i.created_at;
```

### 5.2 recent_executions

```sql
CREATE VIEW recent_executions AS
SELECT 
    o.id,
    o.observation_type,
    json_extract(o.metadata, '$.execution_type') as execution_type,
    json_extract(o.metadata, '$.success') as success,
    json_extract(o.metadata, '$.duration_ms') as duration_ms,
    o.created_at
FROM observations o
WHERE o.observation_type = 'Execution'
ORDER BY o.created_at DESC
LIMIT 100;
```

---

## Summary

### New Tables (9 total)

| Category | Tables |
|----------|--------|
| Agent (4) | `agent_sessions`, `delegations`, `tool_calls`, `checkpoints` |
| Project (4) | `phases`, `issues`, `issue_dependencies`, `decisions` |
| Quality (1) | `quality_gate_configs` |

### Entity Extensions

| Entity | Extension |
|--------|-----------|
| `ObservationType` | Add `Execution` variant |
| `Observation` (Error) | Add `fix_pattern`, `fix_verified` to metadata |

### Size Estimates

| Table | Est. Rows/year | Est. Size |
|-------|----------------|-----------|
| agent_sessions | ~10K | ~2 MB |
| delegations | ~50K | ~5 MB |
| tool_calls | ~500K | ~50 MB |
| checkpoints | ~5K | ~10 MB |
| phases | ~500 | ~100 KB |
| issues | ~5K | ~1 MB |
| decisions | ~2K | ~500 KB |

**Total**: ~70 MB/year

---
*Last updated: 2026-02-03 - aligned with MCB ADR naming patterns*
