# Beads Issue Tracking System - Research Documentation

This directory contains comprehensive documentation of the Beads issue tracking system's data model, storage format, and architecture. This research was conducted to support migration to a relational database.

## Documentation Files

### 1. **beads-data-model.md** (1000 lines, 27 KB)

Complete reference documentation covering:

- Directory structure (.beads/ layout)
- Issue data model (all 50+ fields)
- Related tables (labels, dependencies, comments, events, etc.)
- Database views (ready_issues, blocked_issues)
- JSONL export format with examples
- Configuration file structure
- Git integration and sync workflow
- CLI commands and data requirements
- Indexes and performance characteristics
- Security considerations
- Daemon architecture
- Advanced features (compaction, federation, molecules, gates)
- Migration considerations for relational databases
- Example queries
- Troubleshooting guide

**Use this for**: Complete understanding of Beads data structures, field definitions, relationships, and constraints.

### 2. **beads-quick-reference.md** (196 lines, 5.7 KB)

Quick reference guide with:

- Directory structure overview
- Core tables summary
- JSONL format example
- Key constraints
- CLI commands summary
- Configuration options
- Git integration overview
- Performance characteristics
- Migration checklist
- Important indexes
- Daemon modes
- Advanced features overview
- Troubleshooting commands

**Use this for**: Quick lookup of specific information, CLI commands, or table structures.

### 3. **beads-sql-schema.sql** (342 lines, 12 KB)

Complete SQL schema with:

- All table definitions with constraints
- All indexes (30+ indexes)
- View definitions (ready_issues, blocked_issues)
- Useful query examples (commented)
- Proper foreign key relationships
- Check constraints and defaults

**Use this for**: Creating equivalent tables in a relational database, understanding schema relationships, running queries.

## Key Findings

### Data Model Overview

### Hybrid Storage

- **SQLite** (primary): Fast queries, ACID transactions, daemon-based RPC
- **JSONL** (export): Git-friendly, human-readable, merge-friendly

**Core Entity**: Issues

- 50+ fields covering content, metadata, status, relationships
- Supports multiple issue types (task, bug, feature, epic, molecule, gate, etc.)
- Rich metadata with custom JSON support

### Relationships

- Labels (many-to-many)
- Dependencies (7 types: blocks, discovered-from, parent-child, relates-to, duplicate-of, superseded-by, waits-for)
- Comments (discussion threads)
- Events (audit trail)

### Directory Structure

```text
.beads/
├── beads.db              # SQLite (primary)
├── issues.jsonl          # JSONL export (git-tracked)
├── config.yaml           # Configuration
├── metadata.json         # Database metadata
├── daemon.lock/pid/log   # Daemon runtime
└── export-state/         # Export tracking
```

### Git Integration

1. **bd sync**: Export SQLite → JSONL → git commit → git push
2. **Auto-sync**: Daemon auto-flushes on mutations (debounced)
3. **Merge conflicts**: Intelligent JSONL merge driver
4. **Worktrees**: `.git/beads-worktrees/beads-sync/` for parallel sync

### CLI Commands

| Command | Purpose |
| --------- | --------- |
| `bd create "title"` | Create issue |
| `bd list [--status open]` | List issues |
| `bd show <id>` | Show details |
| `bd update <id> --status in_progress` | Update |
| `bd close <id> --reason "..."` | Close |
| `bd dep add <id> <depends-on>` | Add dependency |
| `bd sync` | Export and push |
| `bd ready` | Show ready issues |
| `bd blocked` | Show blocked issues |

### Database Statistics

- **Tables**: 15 (issues, labels, dependencies, comments, events, config, metadata, dirty_issues, export_hashes, issue_snapshots, compaction_snapshots, repo_mtimes, blocked_issues_cache, child_counters, sqlite_sequence)
- **Views**: 2 (ready_issues, blocked_issues)
- **Indexes**: 30+
- **Typical size**: 1-10 MB per 100-1000 issues
- **Performance**: List < 10ms, Ready issues < 50ms, Dependency traversal < 100ms

### Migration Path to Relational Database

### Tables to Create

1. issues (with all 50+ fields)
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

## Advanced Features

### Compaction

- Reduces issue history size with snapshots
- Multiple compaction levels
- Preserves original content

### Federation

- Multi-repo support (experimental)
- Peer-to-peer issue sharing
- Requires CGO for some features

### Molecules

- Swarms: Multi-polecat coordination
- Patrols: Recurring operations
- Work: Standard work items

### Gates

- Async coordination primitives
- Fanout/join patterns
- Merge-slot for serialized conflict resolution

### Templates

- Issue templates for consistency
- Reusable issue patterns
- Template molecules

## Configuration

Key settings in `config.yaml`:

- `sync-branch`: Git branch for syncing (default: "beads-sync")
- `issue-prefix`: Issue ID prefix (auto-detected)
- `no-db`: Use JSONL only (no SQLite)
- `no-daemon`: Disable daemon
- `no-auto-flush`: Disable auto-export
- `no-auto-import`: Disable auto-import

## Security Considerations

- `.beads/` directory: 700 permissions
- `beads.db`: 600 permissions
- `config.yaml`: 600 permissions
- `issues.jsonl`: 644 permissions (git-tracked)
- Daemon socket: 700 permissions
- No built-in user authentication (relies on git/OS)
- Audit trail via events table

## Performance Characteristics

- List issues by status: < 10ms
- Find ready issues: < 50ms
- Dependency traversal (depth 5): < 100ms
- Full text search: < 200ms
- Database size: 1-10 MB per 100-1000 issues
- WAL files: 1-5 MB (temporary)
- Export to JSONL: < 100ms
- Git commit: < 500ms
- Full sync cycle: < 1s

## Daemon Architecture

### Modes

- **Daemon mode** (default): Background RPC server via Unix socket
- **No-daemon mode**: Direct database access
- **No-db mode**: Load from JSONL, no SQLite

#### Files

- `daemon.pid`: Process ID
- `daemon.lock`: Lock file
- `daemon.log`: Log output
- `bd.sock`: Unix socket for RPC

### Functions

- RPC server for CLI commands
- Auto-flush debouncing
- Database connection pooling
- Concurrent access management

## References

- **Official Repository**: [GitHub.com/steveyegge/beads](https://github.com/steveyegge/beads)
- **Documentation**: [GitHub.com/steveyegge/beads/tree/main/docs](https://github.com/steveyegge/beads/tree/main/docs)
- **Quick Start**: Run `bd quickstart`
- **Help**: Run `bd <command> --help`

## Research Methodology

This documentation was created by:

1. Examining the `.beads/` directory structure
2. Analyzing the SQLite database schema
3. Reviewing JSONL export format with real examples
4. Studying configuration files
5. Documenting CLI commands and their data requirements
6. Extracting database views and indexes
7. Analyzing git integration points
8. Documenting advanced features and constraints

## Next Steps for Migration

1. **Schema Design**: Use beads-sql-schema.sql as template
2. **Data Mapping**: Map Beads fields to target database
3. **Relationship Handling**: Implement dependency graph queries
4. **JSONL Import**: Create import script from JSONL format
5. **Git Integration**: Adapt sync workflow for new database
6. **Testing**: Validate data integrity and query performance
7. **Migration**: Gradual migration with parallel systems
8. **Validation**: Verify all relationships and constraints

---

**Documentation Created**: 2026-02-03
**Beads Version**: Latest (from GitHub.com/steveyegge/beads)
**Database**: SQLite with JSONL export
**Total Documentation**: 1538 lines across 3 files
