<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Beads Data Model - Quick Reference

## Directory Structure

```text
.beads/
├── issues.jsonl          # JSONL export/interchange, not live DB
├── config.yaml           # Configuration
├── metadata.json         # Database metadata
├── embeddeddolt/         # Legacy/solo Dolt data when embedded mode is used
└── export-state/         # Export tracking
```

## Core Tables

### issues (Main Table)

- **id** (PK): Issue ID (e.g., mcb-123)
- **title**: Issue title (≤500 chars)
- **description, design, acceptance_criteria, notes**: Content fields
- **status**: open, in_progress, blocked, deferred, closed, tombstone
- **priority**: 0-4 (0=critical, 4=backlog)
- **issue_type**: task, bug, feature, epic, chore, merge-request, molecule, gate, agent, role, rig, convoy, event
- **assignee, owner, created_by**: People fields
- **created_at, updated_at, closed_at**: Timestamps
- **close_reason**: Reason for closure
- **external_ref**: External reference (gh-123, jira-ABC)
- **labels**: Via separate labels table
- **dependencies**: Via separate dependencies table
- **metadata**: Custom JSON data

### labels (Many-to-Many)

- **issue_id** (FK)
- **label** (String)

### dependencies (Relationships)

- **issue_id** (FK): The dependent issue
- **depends_on_id** (FK): What it depends on
- **type**: blocks, discovered-from, parent-child, relates-to, duplicate-of, superseded-by, waits-for
- **created_at, created_by**: Audit trail

### comments (Discussion)

- **id** (PK)
- **issue_id** (FK)
- **author, text, created_at**

### events (Audit Trail)

- **id** (PK)
- **issue_id** (FK)
- **event_type, actor, old_value, new_value, created_at**

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

- **Closed issues**: Must have closed_at timestamp
- **Dependency cycles**: Prevented
- **Ready issues**: open + not blocked
- **Status transitions**: open → in_progress → closed (or blocked/deferred)

## CLI Commands Summary

| Command | Purpose |
| --------- | --------- |
| `bd create "title"` | Create issue |
| `bd list [--status open]` | List issues |
| `bd show <id>` | Show issue details |
| `bd update <id> --status in_progress` | Update issue |
| `bd close <id> --reason "..."` | Close issue |
| `bd dep add <id> <depends-on>` | Add dependency |
| `bd dep list <id>` | List dependencies |
| `bd dolt push` | Push Dolt commits when a remote is configured |
| `bd dolt pull` | Pull Dolt commits when a remote is configured |
| `bd backup sync` | Push a full Dolt backup to the configured destination |
| `bd ready` | Show ready issues |
| `bd blocked` | Show blocked issues |

## Configuration (config.yaml)

```yaml
# issue-prefix: "mcb"            # Issue prefix

dolt:
  mode: server
  shared-server: true
  host: 127.0.0.1
  port: 3308
  user: root
  database: mcb
  auto-commit: off
```

## Git Integration

1. **Dolt remote sync**: `bd dolt push` / `bd dolt pull` when a remote is configured
2. **Full backup**: `bd backup init <path-or-url>` + `bd backup sync`
3. **JSONL**: `bd export` / `bd import` only for migration/interchange, not normal sync
4. **Multi-agent**: shared-server mode serializes concurrent writers through one Dolt SQL server

## Performance

- List issues: < 10ms
- Find ready issues: < 50ms
- Dependency traversal: < 100ms
- Database size: 1-10 MB per 100-1000 issues

## Migration to Relational DB

### Tables to Create

1. issues (with all fields)
2. labels (many-to-many)
3. dependencies (relationships)
4. comments (discussions)
5. events (audit trail)
6. config (settings)
7. metadata (database metadata)

### Key Relationships

```text
issues (1) ──→ (many) labels
issues (1) ──→ (many) dependencies
issues (1) ──→ (many) comments
issues (1) ──→ (many) events
```

### Data Type Mapping

- Timestamps: DATETIME with timezone
- JSON fields: TEXT (metadata, payload, agent_state, waiters)
- Enums: TEXT (status, issue_type, priority, mol_type, work_type)
- Booleans: INTEGER (0/1)

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

- **Daemon mode** (default): Background RPC server via Unix socket
- **No-daemon mode**: Direct database access
- **Legacy no-db mode**: historical only; do not use for current shared-server coordination
- **Files**: daemon.pid, daemon.lock, daemon.log, bd.sock

## Advanced Features

- **Compaction**: Reduce history size with snapshots
- **Federation**: Multi-repo support (experimental)
- **Molecules**: Swarms, patrols, work items
- **Gates**: Async coordination primitives
- **Templates**: Reusable issue patterns

## Troubleshooting

```bash
bd doctor              # Check health
bd info               # Show database info
bd status             # Show statistics
bd repair             # Fix corrupted database
bd resolve-conflicts  # Resolve git conflicts
```

## References

- **Repository**: [GitHub.com/steveyegge/beads](https://github.com/steveyegge/beads)
- **Docs**: [GitHub.com/steveyegge/beads/tree/main/docs](https://github.com/steveyegge/beads/tree/main/docs)
- **Help**: `bd <command> --help`
