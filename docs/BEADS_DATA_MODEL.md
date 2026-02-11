# Beads Issue Tracking System - Complete Data Model Documentation

## Overview

Beads is an AI-native issue tracking system designed to live in Git repositories. It uses a hybrid storage model combining SQLite (primary) and JSONL (export/sync format) to track issues, dependencies, and metadata.

**Repository**: [GitHub.com/steveyegge/beads](https://github.com/steveyegge/beads)

---

## 1. Directory Structure (.beads/)

```
.beads/
├── beads.db              # SQLite database (primary storage)
├── beads.db-shm         # SQLite shared memory file (WAL mode)
├── beads.db-wal         # SQLite write-ahead log
├── issues.jsonl         # JSONL export (git-tracked, synced via bd sync)
├── interactions.jsonl   # Interaction/comment history (JSONL)
├── config.yaml          # Configuration file (git-tracked)
├── metadata.json        # Database metadata
├── daemon.lock          # Daemon lock file (runtime only)
├── daemon.log           # Daemon log file (runtime only)
├── daemon.pid           # Daemon process ID (runtime only)
├── bd.sock              # Unix socket for daemon RPC (runtime only)
├── last-touched         # Timestamp of last operation
├── .local_version       # Local version tracking
├── .gitignore           # Git ignore rules
├── README.md            # Beads documentation
└── export-state/        # Export state tracking
    └── <hash>.json      # Export state metadata
```

### Git Tracking

-   **Tracked**: `issues.jsonl`, `interactions.jsonl`, `config.yaml`, `README.md`, `.gitignore`
-   **Ignored**: `*.db*`, `daemon.*`, `bd.sock`, `export-state/`, `.sync.lock`

---

## 2. Issue Data Model

### 2.1 Core Issue Fields (SQLite `issues` Table)

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Issue ID (e.g., `mcb-123`, `bd-xyz`) |
| `content_hash` | TEXT | | Hash of issue content for change detection |
| `title` | TEXT | NOT NULL, ≤500 chars | Issue title |
| `description` | TEXT | DEFAULT '' | Full description |
| `design` | TEXT | DEFAULT '' | Design notes/specification |
| `acceptance_criteria` | TEXT | DEFAULT '' | Acceptance criteria |
| `notes` | TEXT | DEFAULT '' | Additional notes |
| `status` | TEXT | DEFAULT 'open' | Status (see 2.2) |
| `priority` | INTEGER | 0-4, DEFAULT 2 | Priority level (0=critical, 4=backlog) |
| `issue_type` | TEXT | DEFAULT 'task' | Type (see 2.3) |
| `assignee` | TEXT | | Assigned person |
| `estimated_minutes` | INTEGER | | Time estimate |
| `created_at` | DATETIME | NOT NULL | Creation timestamp |
| `created_by` | TEXT | DEFAULT '' | Creator name |
| `owner` | TEXT | DEFAULT '' | Owner email/identifier |
| `updated_at` | DATETIME | NOT NULL | Last update timestamp |
| `closed_at` | DATETIME | | Closure timestamp (NULL if open) |
| `closed_by_session` | TEXT | DEFAULT '' | Session ID that closed it |
| `close_reason` | TEXT | DEFAULT '' | Reason for closure |
| `external_ref` | TEXT | UNIQUE | External reference (e.g., `gh-123`, `jira-ABC`) |
| `compaction_level` | INTEGER | DEFAULT 0 | Compaction level for history |
| `compacted_at` | DATETIME | | Compaction timestamp |
| `compacted_at_commit` | TEXT | | Git commit of compaction |
| `original_size` | INTEGER | | Original size before compaction |
| `deleted_at` | DATETIME | | Deletion timestamp |
| `deleted_by` | TEXT | DEFAULT '' | Who deleted it |
| `delete_reason` | TEXT | DEFAULT '' | Reason for deletion |
| `original_type` | TEXT | DEFAULT '' | Original type before change |
| `sender` | TEXT | DEFAULT '' | Sender (for messages) |
| `ephemeral` | INTEGER | DEFAULT 0 | 1 if ephemeral (not exported) |
| `pinned` | INTEGER | DEFAULT 0 | 1 if pinned |
| `is_template` | INTEGER | DEFAULT 0 | 1 if template molecule |
| `crystallizes` | INTEGER | DEFAULT 0 | 1 if crystallizes work |
| `mol_type` | TEXT | DEFAULT '' | Molecule type (swarm, patrol, work) |
| `work_type` | TEXT | DEFAULT 'Mutex' | Work type (Mutex, open_competition) |
| `quality_score` | REAL | 0.0-1.0 | Quality score (set by refineries) |
| `source_system` | TEXT | DEFAULT '' | Federation source system |
| `event_kind` | TEXT | DEFAULT '' | Event kind (for event issues) |
| `actor` | TEXT | DEFAULT '' | Actor URI (for events) |
| `target` | TEXT | DEFAULT '' | Target URI (for events) |
| `payload` | TEXT | DEFAULT '' | Event payload (JSON) |
| `source_repo` | TEXT | DEFAULT '.' | Source repository |
| `await_type` | TEXT | | Await type (gate coordination) |
| `await_id` | TEXT | | Await ID (gate coordination) |
| `timeout_ns` | INTEGER | | Timeout in nanoseconds |
| `waiters` | TEXT | | Waiters list (JSON) |
| `hook_bead` | TEXT | DEFAULT '' | Hook bead ID |
| `role_bead` | TEXT | DEFAULT '' | Role bead ID |
| `agent_state` | TEXT | DEFAULT '' | Agent state (JSON) |
| `last_activity` | DATETIME | | Last activity timestamp |
| `role_type` | TEXT | DEFAULT '' | Role type |
| `rig` | TEXT | DEFAULT '' | Rig name (partition) |
| `due_at` | DATETIME | | Due date/time |
| `defer_until` | DATETIME | | Defer until date/time |
| `metadata` | TEXT | DEFAULT '{}' | Custom metadata (JSON) |

### 2.2 Status Values

-   `open` - Issue is open and available
-   `in_progress` - Currently being worked on
-   `blocked` - Blocked by dependencies
-   `deferred` - Deferred until later
-   `closed` - Completed/resolved
-   `tombstone` - Deleted but preserved for history

### 2.3 Issue Types

-   `task` - Standard task
-   `bug` - Bug report
-   `feature` - Feature request
-   `epic` - Epic (large feature)
-   `chore` - Maintenance work
-   `merge-request` / `mr` - Merge request
-   `molecule` - Molecule (swarm/patrol/work)
-   `gate` - Async coordination gate
-   `agent` - Agent state tracking
-   `role` - Role definition
-   `rig` - Rig (partition) definition
-   `convoy` - Convoy (group coordination)
-   `event` - Event tracking

### 2.4 Priority Levels

-   `0` / `P0` - Critical (highest)
-   `1` / `P1` - High
-   `2` / `P2` - Medium (default)
-   `3` / `P3` - Low
-   `4` / `P4` - Backlog (lowest)

---

## 3. Related Tables

### 3.1 Labels Table

```sql
CREATE TABLE labels (
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (issue_id, label),
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Many-to-many relationship for issue labels
**Example**: `mcb-123` → `["phase-6", "memory", "v0.2.0"]`

### 3.2 Dependencies Table

```sql
CREATE TABLE dependencies (
    issue_id TEXT NOT NULL,
    depends_on_id TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'blocks',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    metadata TEXT,
    thread_id TEXT,
    PRIMARY KEY (issue_id, depends_on_id, type),
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Tracks dependencies between issues

**Dependency Types**:

-   `blocks` - Issue A blocks Issue B (B depends on A)
-   `discovered-from` - Discovered from another issue
-   `parent-child` - Hierarchical parent-child relationship
-   `relates-to` - Bidirectional relationship
-   `duplicate-of` - Duplicate relationship
-   `superseded-by` - Superseded relationship
-   `waits-for` - Gate coordination (fanout)

**Example**:

```
mcb-7xi blocks mcb-6zi    (mcb-6zi depends on mcb-7xi)
mcb-hv1 blocks mcb-7xi    (mcb-7xi blocks mcb-hv1)
```

### 3.3 Comments Table

```sql
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    author TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Comments/discussions on issues

### 3.4 Events Table

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    actor TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    comment TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Audit trail of changes to issues

**Event Types**: status_changed, priority_changed, assigned, etc.

### 3.5 Configuration Table

```sql
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**Purpose**: Stores configuration key-value pairs

### 3.6 Metadata Table

```sql
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**Purpose**: Database-level metadata

### 3.7 Dirty Issues Table

```sql
CREATE TABLE dirty_issues (
    issue_id TEXT PRIMARY KEY,
    marked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content_hash TEXT,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Tracks issues that need JSONL export

### 3.8 Export Hashes Table

```sql
CREATE TABLE export_hashes (
    issue_id TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    exported_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Tracks exported issue hashes for sync detection

### 3.9 Issue Snapshots Table

```sql
CREATE TABLE issue_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    snapshot_time DATETIME NOT NULL,
    compaction_level INTEGER NOT NULL,
    original_size INTEGER NOT NULL,
    compressed_size INTEGER NOT NULL,
    original_content TEXT NOT NULL,
    archived_events TEXT,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Stores compressed snapshots of issue history

### 3.10 Compaction Snapshots Table

```sql
CREATE TABLE compaction_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    compaction_level INTEGER NOT NULL,
    snapshot_json BLOB NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);
```

**Purpose**: Stores compacted issue snapshots

### 3.11 Repository MTimes Table

```sql
CREATE TABLE repo_mtimes (
    repo_path TEXT PRIMARY KEY,
    jsonl_path TEXT NOT NULL,
    mtime_ns INTEGER NOT NULL,
    last_checked DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Purpose**: Tracks JSONL file modification times for multi-repo sync

---

## 4. Views

### 4.1 ready_issues View

```sql
CREATE VIEW ready_issues AS
    WITH RECURSIVE
      blocked_directly AS (
        SELECT DISTINCT d.issue_id
        FROM dependencies d
        JOIN issues blocker ON d.depends_on_id = blocker.id
        WHERE d.type = 'blocks'
          AND blocker.status IN ('open', 'in_progress', 'blocked', 'deferred')
      ),
      blocked_transitively AS (
        SELECT issue_id, 0 as depth
        FROM blocked_directly
        UNION ALL
        SELECT d.issue_id, bt.depth + 1
        FROM blocked_transitively bt
        JOIN dependencies d ON d.depends_on_id = bt.issue_id
        WHERE d.type = 'parent-child'
          AND bt.depth < 50
      )
    SELECT i.*
    FROM issues i
    WHERE i.status = 'open'
      AND NOT EXISTS (
        SELECT 1 FROM blocked_transitively WHERE issue_id = i.id
      );
```

**Purpose**: Returns issues that are ready to work on (open, not blocked)

### 4.2 blocked_issues View

```sql
CREATE VIEW blocked_issues AS
    SELECT
        i.*,
        COUNT(d.depends_on_id) as blocked_by_count
    FROM issues i
    JOIN dependencies d ON i.id = d.issue_id
    JOIN issues blocker ON d.depends_on_id = blocker.id
    WHERE i.status IN ('open', 'in_progress', 'blocked', 'deferred')
      AND d.type = 'blocks'
      AND blocker.status IN ('open', 'in_progress', 'blocked', 'deferred')
    GROUP BY i.id;
```

**Purpose**: Returns issues that are blocked with count of blockers

---

## 5. JSONL Export Format

### 5.1 File Structure

**File**: `.beads/issues.jsonl`

-   One JSON object per line (JSONL format)
-   Each line is a complete issue record
-   Synced to git via `bd sync` command

### 5.2 JSONL Issue Record Example

```json
{
  "id": "mcb-7xi",
  "title": "MEM-04a: Implement memory (action=list, resource=observation) tool (token-efficient index)",
  "status": "closed",
  "priority": 2,
  "issue_type": "task",
  "owner": "marlonsc@gmail.com",
  "created_at": "2026-02-02T15:04:10.495335964-03:00",
  "created_by": "Marlon Costa",
  "updated_at": "2026-02-02T16:51:40.515480751-03:00",
  "closed_at": "2026-02-02T16:51:40.515480751-03:00",
  "close_reason": "Implementada ferramenta memory (action=list, resource=observation)",
  "labels": ["memory", "phase-6"],
  "dependencies": [
    {
      "issue_id": "mcb-7xi",
      "depends_on_id": "mcb-95z",
      "type": "blocks",
      "created_at": "2026-02-02T15:04:23.950531335-03:00",
      "created_by": "Marlon Costa"
    }
  ]
}
```

### 5.3 JSONL Fields

-   All fields from the `issues` table
-   `labels` - Array of label strings
-   `dependencies` - Array of dependency objects with:
  -   `issue_id` - The issue
  -   `depends_on_id` - What it depends on
  -   `type` - Dependency type
  -   `created_at` - When dependency was created
  -   `created_by` - Who created it

---

## 6. Configuration File (config.yaml)

```yaml

# Issue prefix for this repository

# issue-prefix: "mcb"

# Use no-db mode (load from JSONL, no SQLite)

# no-db: false

# Disable daemon for RPC communication

# no-daemon: false

# Disable auto-flush of database to JSONL

# no-auto-flush: false

# Disable auto-import from JSONL when newer

# no-auto-import: false

# Enable JSON output by default

# json: false

# Default actor for audit trails

# actor: ""

# Path to database

# db: ""

# Auto-start daemon if not running

# auto-start-daemon: true

# Debounce interval for auto-flush

# flush-debounce: "5s"

# Git branch for beads commits (IMPORTANT for team projects)
sync-branch: "beads-sync"

# Multi-repo configuration (experimental)

# repos:

#   primary: "."

#   additional:

#     - ~/beads-planning

#     - ~/work-planning
```

---

## 7. Metadata Files

### 7.1 metadata.JSON

```json
{
  "database": "beads.db",
  "jsonl_export": "issues.jsonl"
}
```

### 7.2 export-state/\<hash\>.JSON

```json
{
  "worktree_root": "/home/marlonsc/mcb/.git/beads-worktrees/beads-sync",
  "last_export_commit": "e3483ac7c6076a88b271346f6d7badd4b7b5687b",
  "last_export_time": "2026-01-31T19:54:29.726099827-03:00",
  "jsonl_hash": "d6bb4b6a22dd4757f1483f807b8c7ea5e0ef0cb9331b9eae809fa7131128a20c"
}
```

---

## 8. Git Integration

### 8.1 Sync Workflow

1. **bd sync** command:

-   Exports SQLite database to `.beads/issues.jsonl`
-   Commits changes to git
-   Pushes to remote on `sync-branch` (default: `beads-sync`)

1. **Auto-sync**:

-   Daemon monitors database changes
-   Auto-flushes to JSONL on mutations
-   Debounced to prevent excessive writes

1. **Merge Conflict Resolution**:

-   Beads provides intelligent JSONL merge driver
-   Handles concurrent edits gracefully
-   Preserves dependency integrity

### 8.2 Worktrees

-   Beads uses git worktrees for sync operations
-   Location: `.git/beads-worktrees/beads-sync/`
-   Allows parallel sync without blocking main branch

### 8.3 Hooks

-   Git hooks auto-call `bd prime` for context recovery
-   Hooks auto-sync on commits
-   Can be managed with `bd hooks` command

---

## 9. CLI Commands and Data Requirements

### 9.1 Create Issue

```bash
bd create "Title" [flags]
```

**Data Requirements**:

-   `title` (required)
-   `--description` / `-d` - Description text
-   `--design` - Design notes
-   `--acceptance` - Acceptance criteria
-   `--notes` - Additional notes
-   `--type` / `-t` - Issue type (default: task)
-   `--priority` / `-p` - Priority 0-4 (default: 2)
-   `--assignee` / `-a` - Assignee
-   `--estimate` / `-e` - Time estimate in minutes
-   `--labels` / `-l` - Labels (comma-separated)
-   `--due` - Due date (formats: +6h, +1d, tomorrow, 2025-01-15)
-   `--defer` - Defer until date
-   `--deps` - Dependencies (format: `type:id` or `id`)
-   `--parent` - Parent issue ID
-   `--external-ref` - External reference (gh-123, jira-ABC)
-   `--rig` - Rig/partition name
-   `--prefix` - Issue prefix override
-   `--id` - Explicit issue ID
-   `--ephemeral` - Create as ephemeral (not exported)
-   `--pinned` - Create as pinned
-   `--mol-type` - Molecule type (swarm, patrol, work)

### 9.2 Update Issue

```bash
bd update <id> [flags]
```

**Data Requirements**:

-   `--status` - New status
-   `--priority` - New priority
-   `--assignee` - New assignee
-   `--title` - New title
-   `--description` - New description
-   `--design` - New design notes
-   `--acceptance` - New acceptance criteria
-   `--notes` - New notes
-   `--append-notes` - Append to notes
-   `--due` - New due date
-   `--defer` - New defer date
-   `--estimate` - New time estimate
-   `--labels` - New labels

### 9.3 List Issues

```bash
bd list [flags]
```

**Filter Options**:

-   `--status` - Filter by status
-   `--priority` / `-p` - Filter by priority
-   `--assignee` / `-a` - Filter by assignee
-   `--label` / `-l` - Filter by labels (AND)
-   `--label-any` - Filter by labels (OR)
-   `--type` / `-t` - Filter by type
-   `--title` - Filter by title substring
-   `--description` - Filter by description
-   `--created-after` / `--created-before` - Date range
-   `--updated-after` / `--updated-before` - Date range
-   `--closed-after` / `--closed-before` - Date range
-   `--due-after` / `--due-before` - Date range
-   `--defer-after` / `--defer-before` - Date range
-   `--ready` - Show only ready issues
-   `--parent` - Filter by parent issue
-   `--pinned` - Show only pinned
-   `--deferred` - Show only deferred
-   `--overdue` - Show only overdue

**Output Options**:

-   `--json` - JSON output
-   `--long` - Detailed output
-   `--pretty` / `--tree` - Tree format
-   `--sort` - Sort by field (priority, created, updated, closed, status, id, title, type, assignee)
-   `--limit` / `-n` - Limit results (default: 50)
-   `--reverse` / `-r` - Reverse sort order

### 9.4 Show Issue

```bash
bd show <id> [flags]
```

**Options**:

-   `--short` - Compact one-line output
-   `--children` - Show only children
-   `--refs` - Show reverse references
-   `--thread` - Show full conversation thread
-   `--as-of` - Show at specific commit (requires Dolt)
-   `--local-time` - Show in local time
-   `--json` - JSON output

### 9.5 Dependency Management

```bash
bd dep add <issue> <depends-on>
bd dep remove <issue> <depends-on>
bd dep list <issue>
bd dep tree <issue>
bd dep cycles
bd dep <blocker> --blocks <blocked>
```

**Data Requirements**:

-   `issue` - The dependent issue
-   `depends-on` - The issue it depends on
-   `--type` - Dependency type (blocks, discovered-from, parent-child, relates-to, duplicate-of, superseded-by, waits-for)

### 9.6 Close Issue

```bash
bd close <id> [flags]
```

**Data Requirements**:

-   `--reason` - Reason for closure
-   `--json` - JSON output

### 9.7 Sync with Git

```bash
bd sync [flags]
```

**Options**:

-   `--status` - Check sync status without syncing
-   `--json` - JSON output

---

## 10. Indexes

Key indexes for performance:

```sql
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_assignee ON issues(assignee);
CREATE INDEX idx_issues_created_at ON issues(created_at);
CREATE INDEX idx_issues_updated_at ON issues(updated_at);
CREATE INDEX idx_issues_due_at ON issues(due_at);
CREATE INDEX idx_issues_defer_until ON issues(defer_until);
CREATE INDEX idx_issues_status_priority ON issues(status, priority);
CREATE INDEX idx_labels_label ON labels(label);
CREATE INDEX idx_labels_label_issue ON labels(label, issue_id);
CREATE INDEX idx_dependencies_issue_id ON dependencies(issue_id);
CREATE INDEX idx_dependencies_depends_on ON dependencies(depends_on_id);
CREATE INDEX idx_dependencies_type ON dependencies(type);
CREATE INDEX idx_dependencies_depends_on_type ON dependencies(depends_on_id, type);
CREATE INDEX idx_comments_issue ON comments(issue_id);
CREATE INDEX idx_events_issue ON events(issue_id);
```

---

## 11. Storage Architecture

### 11.1 Hybrid Storage Model

**SQLite (Primary)**:

-   Fast queries and filtering
-   ACID transactions
-   Daemon-based RPC access
-   WAL mode for concurrent access

**JSONL (Export)**:

-   Git-friendly format
-   Human-readable
-   Merge-friendly
-   Source of truth for sync

### 11.2 Sync Flow

```
SQLite Database
    ↓ (bd sync)
JSONL Export
    ↓ (git commit)
Git Repository
    ↓ (git push)
Remote Repository
```

### 11.3 Import Flow

```
Remote Repository
    ↓ (git pull)
Git Repository
    ↓ (auto-import if newer)
JSONL File
    ↓ (daemon loads)
SQLite Database
```

---

## 12. Key Constraints and Rules

### 12.1 Issue ID Format

-   Format: `<prefix>-<alphanumeric>`
-   Example: `mcb-123`, `bd-xyz`, `beads-abc`
-   Prefix configured in `config.yaml` or auto-detected from directory name

### 12.2 Status Transitions

-   `open` → `in_progress` → `closed`
-   `open` → `blocked` (when dependencies exist)
-   `open` → `deferred` (when defer_until set)
-   `closed` → `open` (reopen)
-   Any → `tombstone` (deletion)

### 12.3 Dependency Constraints

-   Circular dependencies detected and prevented
-   Transitive blocking: if A blocks B and B blocks C, then A indirectly blocks C
-   Ready issues: open issues with no blocking dependencies

### 12.4 Closed Issue Constraint

```sql
CHECK (
    (status = 'closed' AND closed_at IS NOT NULL) OR
    (status = 'tombstone') OR
    (status NOT IN ('closed', 'tombstone') AND closed_at IS NULL)
)
```

-   Closed issues must have `closed_at` timestamp
-   Tombstone issues may retain `closed_at` from before deletion

---

## 13. Migration Considerations for Relational Database

### 13.1 Key Tables to Create

1. **issues** - Main issue table (see 2.1)
2. **labels** - Many-to-many label mapping
3. **dependencies** - Issue relationships
4. **comments** - Discussion threads
5. **events** - Audit trail
6. **config** - Configuration storage
7. **metadata** - Database metadata

### 13.2 Relationships

```
issues (1) ──→ (many) labels
issues (1) ──→ (many) dependencies
issues (1) ──→ (many) comments
issues (1) ──→ (many) events
issues (1) ──→ (many) issue_snapshots
```

### 13.3 Data Types

-   **Timestamps**: Use DATETIME or TIMESTAMP with timezone support
-   **JSON fields**: Store as TEXT (metadata, payload, agent_state, waiters)
-   **Enums**: Store as TEXT (status, issue_type, priority, mol_type, work_type, event_kind, await_type)
-   **Booleans**: Store as INTEGER (0/1) for SQLite compatibility

### 13.4 Indexes to Maintain

-   Status, priority, assignee for filtering
-   Created/updated/closed dates for time-based queries
-   Dependencies for graph traversal
-   Labels for tag-based filtering
-   External references for integration lookups

---

## 14. Example Queries

### 14.1 Find Ready Issues

```sql
SELECT * FROM ready_issues WHERE status = 'open' ORDER BY priority ASC;
```

### 14.2 Find Blocked Issues

```sql
SELECT * FROM blocked_issues ORDER BY blocked_by_count DESC;
```

### 14.3 Find Issues by Label

```sql
SELECT DISTINCT i.* FROM issues i
JOIN labels l ON i.id = l.issue_id
WHERE l.label = 'phase-6'
ORDER BY i.priority ASC;
```

### 14.4 Find Dependency Chain

```sql
WITH RECURSIVE deps AS (
  SELECT issue_id, depends_on_id, 1 as depth
  FROM dependencies
  WHERE issue_id = 'mcb-123'
  UNION ALL
  SELECT d.issue_id, d.depends_on_id, deps.depth + 1
  FROM dependencies d
  JOIN deps ON d.issue_id = deps.depends_on_id
  WHERE deps.depth < 10
)
SELECT DISTINCT depends_on_id FROM deps;
```

### 14.5 Find Overdue Issues

```sql
SELECT * FROM issues
WHERE due_at < CURRENT_TIMESTAMP
  AND status != 'closed'
ORDER BY due_at ASC;
```

---

## 15. Performance Characteristics

### 15.1 Typical Query Times

-   List issues by status: < 10ms
-   Find ready issues: < 50ms
-   Dependency traversal (depth 5): < 100ms
-   Full text search: < 200ms

### 15.2 Database Size

-   Typical: 1-10 MB for 100-1000 issues
-   With history: 10-100 MB
-   WAL files: 1-5 MB (temporary)

### 15.3 Sync Performance

-   Export to JSONL: < 100ms
-   Git commit: < 500ms
-   Full sync cycle: < 1s

---

## 16. Security Considerations

### 16.1 File Permissions

-   `.beads/` directory: 700 (rwx------)
-   `beads.db`: 600 (rw-------)
-   `config.yaml`: 600 (rw-------)
-   `issues.jsonl`: 644 (rw-r--r--) - git tracked

### 16.2 Sensitive Data

-   Passwords/tokens: Store in environment variables, not in issues
-   External refs: Can expose integration URLs
-   Metadata: May contain sensitive information

### 16.3 Access Control

-   Daemon socket: Unix socket with 700 permissions
-   No built-in user authentication (relies on git/OS)
-   Audit trail via events table

---

## 17. Daemon Architecture

### 17.1 Daemon Files

-   `daemon.pid` - Process ID
-   `daemon.lock` - Lock file
-   `daemon.log` - Log output
-   `bd.sock` - Unix socket for RPC

### 17.2 Daemon Functions

-   RPC server for CLI commands
-   Auto-flush debouncing
-   Database connection pooling
-   Concurrent access management

### 17.3 Daemon Modes

-   **Daemon mode** (default): Background RPC server
-   **No-daemon mode**: Direct database access
-   **No-db mode**: Load from JSONL, no SQLite

---

## 18. Advanced Features

### 18.1 Compaction

-   Reduces issue history size
-   Creates snapshots at different levels
-   Preserves original content
-   Enables long-term storage efficiency

### 18.2 Federation

-   Multi-repo support (experimental)
-   Peer-to-peer issue sharing
-   Requires CGO for some features

### 18.3 Molecules

-   Swarms: Multi-polecat coordination
-   Patrols: Recurring operations
-   Work: Standard work items

### 18.4 Gates

-   Async coordination primitives
-   Fanout/join patterns
-   Merge-slot for serialized conflict resolution

### 18.5 Templates

-   Issue templates for consistency
-   Reusable issue patterns
-   Template molecules

---

## 19. Troubleshooting

### 19.1 Common Issues

**Database locked**:

-   Check daemon status: `bd info`
-   Restart daemon: `bd daemon restart`
-   Use `--lock-timeout` flag

**Sync conflicts**:

-   Run `bd resolve-conflicts`
-   Manual merge if needed
-   Check git status

**Stale data**:

-   Run `bd sync` to export
-   Use `--allow-stale` flag to override

**Corrupted database**:

-   Run `bd repair` to clean orphaned references
-   Restore from git: `bd restore <issue-id>`

### 19.2 Diagnostic Commands

```bash
bd doctor              # Check installation health
bd info               # Show database and daemon info
bd status             # Show statistics
bd preflight          # PR readiness checklist
```

---

## 20. References

-   **Official Repository**: [GitHub.com/steveyegge/beads](https://github.com/steveyegge/beads)
-   **Documentation**: [GitHub.com/steveyegge/beads/tree/main/docs](https://github.com/steveyegge/beads/tree/main/docs)
-   **Quick Start**: Run `bd quickstart`
-   **Help**: Run `bd <command> --help`
