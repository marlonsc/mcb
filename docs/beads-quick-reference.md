# Beads Data Model - Quick Reference

## Directory Structure

```
.beads/
├── beads.db              # SQLite (primary storage)
├── issues.jsonl          # JSONL export (git-tracked)
├── config.yaml           # Configuration
├── metadata.json         # Database metadata
└── export-state/         # Export tracking
```

## Core Tables

### issues (Main Table)

-   **id** (PK): Issue ID (e.g., mcb-123)
-   **title**: Issue title (≤500 chars)
-   **description, design, acceptance_criteria, notes**: Content fields
-   **status**: open, in_progress, blocked, deferred, closed, tombstone
-   **priority**: 0-4 (0=critical, 4=backlog)
-   **issue_type**: task, bug, feature, epic, chore, merge-request, molecule, gate, agent, role, rig, convoy, event
-   **assignee, owner, created_by**: People fields
-   **created_at, updated_at, closed_at**: Timestamps
-   **close_reason**: Reason for closure
-   **external_ref**: External reference (gh-123, jira-ABC)
-   **labels**: Via separate labels table
-   **dependencies**: Via separate dependencies table
-   **metadata**: Custom JSON data

### labels (Many-to-Many)

-   **issue_id** (FK)
-   **label** (String)

### dependencies (Relationships)

-   **issue_id** (FK): The dependent issue
-   **depends_on_id** (FK): What it depends on
-   **type**: blocks, discovered-from, parent-child, relates-to, duplicate-of, superseded-by, waits-for
-   **created_at, created_by**: Audit trail

### comments (Discussion)

-   **id** (PK)
-   **issue_id** (FK)
-   **author, text, created_at**

### events (Audit Trail)

-   **id** (PK)
-   **issue_id** (FK)
-   **event_type, actor, old_value, new_value, created_at**

## Views

### ready_issues

Issues that are open and not blocked by any dependencies

### blocked_issues

Issues that are blocked with count of blockers

## JSONL Format

One JSON object per line:

```json
{
  "id": "mcb-123",
  "title": "...",
  "status": "open",
  "priority": 2,
  "issue_type": "task",
  "labels": ["label1", "label2"],
  "dependencies": [
    {
      "issue_id": "mcb-123",
      "depends_on_id": "mcb-456",
      "type": "blocks",
      "created_at": "...",
      "created_by": "..."
    }
  ],
  ...
}
```

## Key Constraints

-   **Closed issues**: Must have closed_at timestamp
-   **Dependency cycles**: Prevented
-   **Ready issues**: open + not blocked
-   **Status transitions**: open → in_progress → closed (or blocked/deferred)

## CLI Commands Summary

| Command | Purpose |
|---------|---------|
| `bd create "title"` | Create issue |
| `bd list [--status open]` | List issues |
| `bd show <id>` | Show issue details |
| `bd update <id> --status in_progress` | Update issue |
| `bd close <id> --reason "..."` | Close issue |
| `bd dep add <id> <depends-on>` | Add dependency |
| `bd dep list <id>` | List dependencies |
| `bd sync` | Export to JSONL and push to git |
| `bd ready` | Show ready issues |
| `bd blocked` | Show blocked issues |

## Configuration (config.yaml)

```yaml
sync-branch: "beads-sync"        # Git branch for syncing
# issue-prefix: "mcb"            # Issue prefix
# no-db: false                   # Use JSONL only
# no-daemon: false               # Disable daemon
# no-auto-flush: false           # Disable auto-export
# no-auto-import: false          # Disable auto-import
```

## Git Integration

1.  **bd sync**: Export SQLite → JSONL → git commit → git push
2.  **Auto-sync**: Daemon auto-flushes on mutations (debounced)
3.  **Merge conflicts**: Intelligent JSONL merge driver
4.  **Worktrees**: `.git/beads-worktrees/beads-sync/` for parallel sync

## Performance

-   List issues: < 10ms
-   Find ready issues: < 50ms
-   Dependency traversal: < 100ms
-   Database size: 1-10 MB per 100-1000 issues

## Migration to Relational DB

### Tables to Create

1.  issues (with all fields)
2.  labels (many-to-many)
3.  dependencies (relationships)
4.  comments (discussions)
5.  events (audit trail)
6.  config (settings)
7.  metadata (database metadata)

### Key Relationships

```
issues (1) ──→ (many) labels
issues (1) ──→ (many) dependencies
issues (1) ──→ (many) comments
issues (1) ──→ (many) events
```

### Data Type Mapping

-   Timestamps: DATETIME with timezone
-   JSON fields: TEXT (metadata, payload, agent_state, waiters)
-   Enums: TEXT (status, issue_type, priority, mol_type, work_type)
-   Booleans: INTEGER (0/1)

## Important Indexes

```sql
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_assignee ON issues(assignee);
CREATE INDEX idx_issues_created_at ON issues(created_at);
CREATE INDEX idx_issues_updated_at ON issues(updated_at);
CREATE INDEX idx_dependencies_issue_id ON dependencies(issue_id);
CREATE INDEX idx_dependencies_depends_on ON dependencies(depends_on_id);
CREATE INDEX idx_labels_label ON labels(label);
```

## Daemon Architecture

-   **Daemon mode** (default): Background RPC server via Unix socket
-   **No-daemon mode**: Direct database access
-   **No-db mode**: Load from JSONL, no SQLite
-   **Files**: daemon.pid, daemon.lock, daemon.log, bd.sock

## Advanced Features

-   **Compaction**: Reduce history size with snapshots
-   **Federation**: Multi-repo support (experimental)
-   **Molecules**: Swarms, patrols, work items
-   **Gates**: Async coordination primitives
-   **Templates**: Reusable issue patterns

## Troubleshooting

```bash
bd doctor              # Check health
bd info               # Show database info
bd status             # Show statistics
bd repair             # Fix corrupted database
bd resolve-conflicts  # Resolve git conflicts
```

## References

-   **Repository**: [GitHub.com/steveyegge/beads](https://github.com/steveyegge/beads)
-   **Docs**: [GitHub.com/steveyegge/beads/tree/main/docs](https://github.com/steveyegge/beads/tree/main/docs)
-   **Help**: `bd <command> --help`
