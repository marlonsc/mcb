-- Beads Issue Tracking System - Complete SQL Schema
-- This is the SQLite schema used by Beads for persistent storage

-- Main issues table
CREATE TABLE issues (
    id TEXT PRIMARY KEY,
    content_hash TEXT,
    title TEXT NOT NULL CHECK(length(title) <= 500),
    description TEXT NOT NULL DEFAULT '',
    design TEXT NOT NULL DEFAULT '',
    acceptance_criteria TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    priority INTEGER NOT NULL DEFAULT 2 CHECK(priority >= 0 AND priority <= 4),
    issue_type TEXT NOT NULL DEFAULT 'task',
    assignee TEXT,
    estimated_minutes INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT DEFAULT '',
    owner TEXT DEFAULT '',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at DATETIME,
    closed_by_session TEXT DEFAULT '',
    external_ref TEXT,
    compaction_level INTEGER DEFAULT 0,
    compacted_at DATETIME,
    compacted_at_commit TEXT,
    original_size INTEGER,
    deleted_at DATETIME,
    deleted_by TEXT DEFAULT '',
    delete_reason TEXT DEFAULT '',
    original_type TEXT DEFAULT '',
    sender TEXT DEFAULT '',
    ephemeral INTEGER DEFAULT 0,
    pinned INTEGER DEFAULT 0,
    is_template INTEGER DEFAULT 0,
    crystallizes INTEGER DEFAULT 0,
    mol_type TEXT DEFAULT '',
    work_type TEXT DEFAULT 'mutex',
    quality_score REAL,
    source_system TEXT DEFAULT '',
    event_kind TEXT DEFAULT '',
    actor TEXT DEFAULT '',
    target TEXT DEFAULT '',
    payload TEXT DEFAULT '',
    source_repo TEXT DEFAULT '.',
    close_reason TEXT DEFAULT '',
    await_type TEXT,
    await_id TEXT,
    timeout_ns INTEGER,
    waiters TEXT,
    hook_bead TEXT DEFAULT '',
    role_bead TEXT DEFAULT '',
    agent_state TEXT DEFAULT '',
    last_activity DATETIME,
    role_type TEXT DEFAULT '',
    rig TEXT DEFAULT '',
    due_at DATETIME,
    defer_until DATETIME,
    metadata TEXT NOT NULL DEFAULT '{}',
    CHECK (
        (status = 'closed' AND closed_at IS NOT NULL) OR
        (status = 'tombstone') OR
        (status NOT IN ('closed', 'tombstone') AND closed_at IS NULL)
    )
);

-- Labels (many-to-many relationship)
CREATE TABLE labels (
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (issue_id, label),
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Dependencies between issues
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

-- Comments/discussions on issues
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    author TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Audit trail of changes
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

-- Configuration key-value store
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Database metadata
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Tracks issues that need JSONL export
CREATE TABLE dirty_issues (
    issue_id TEXT PRIMARY KEY,
    marked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content_hash TEXT,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Tracks exported issue hashes for sync detection
CREATE TABLE export_hashes (
    issue_id TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    exported_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Issue history snapshots
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

-- Compacted issue snapshots
CREATE TABLE compaction_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    compaction_level INTEGER NOT NULL,
    snapshot_json BLOB NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Repository modification times (for multi-repo sync)
CREATE TABLE repo_mtimes (
    repo_path TEXT PRIMARY KEY,
    jsonl_path TEXT NOT NULL,
    mtime_ns INTEGER NOT NULL,
    last_checked DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Blocked issues cache
CREATE TABLE blocked_issues_cache (
    issue_id TEXT NOT NULL,
    PRIMARY KEY (issue_id),
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Child counters for hierarchical issues
CREATE TABLE child_counters (
    parent_id TEXT PRIMARY KEY,
    last_child INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (parent_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Issues table indexes
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_assignee ON issues(assignee);
CREATE INDEX idx_issues_created_at ON issues(created_at);
CREATE INDEX idx_issues_updated_at ON issues(updated_at);
CREATE INDEX idx_issues_due_at ON issues(due_at);
CREATE INDEX idx_issues_defer_until ON issues(defer_until);
CREATE INDEX idx_issues_status_priority ON issues(status, priority);
CREATE INDEX idx_issues_external_ref ON issues(external_ref);
CREATE INDEX idx_issues_source_repo ON issues(source_repo);
CREATE INDEX idx_issues_deleted_at ON issues(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_issues_ephemeral ON issues(ephemeral) WHERE ephemeral = 1;
CREATE INDEX idx_issues_sender ON issues(sender) WHERE sender != '';
CREATE INDEX idx_issues_pinned ON issues(pinned) WHERE pinned = 1;
CREATE INDEX idx_issues_is_template ON issues(is_template) WHERE is_template = 1;
CREATE INDEX idx_issues_gate ON issues(issue_type) WHERE issue_type = 'gate';
CREATE UNIQUE INDEX idx_issues_external_ref_unique ON issues(external_ref) WHERE external_ref IS NOT NULL;

-- Labels indexes
CREATE INDEX idx_labels_label ON labels(label);
CREATE INDEX idx_labels_label_issue ON labels(label, issue_id);

-- Dependencies indexes
CREATE INDEX idx_dependencies_issue_id ON dependencies(issue_id);
CREATE INDEX idx_dependencies_depends_on ON dependencies(depends_on_id);
CREATE INDEX idx_dependencies_type ON dependencies(type);
CREATE INDEX idx_dependencies_depends_on_type ON dependencies(depends_on_id, type);
CREATE INDEX idx_dependencies_depends_on_type_issue ON dependencies(depends_on_id, type, issue_id);
CREATE INDEX idx_dependencies_issue_type ON dependencies(issue_id, type);

-- Comments indexes
CREATE INDEX idx_comments_issue ON comments(issue_id);
CREATE INDEX idx_comments_created_at ON comments(created_at);

-- Events indexes
CREATE INDEX idx_events_issue ON events(issue_id);
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_events_issue_type ON events(issue_id, event_type);

-- Dirty issues indexes
CREATE INDEX idx_dirty_issues_marked_at ON dirty_issues(marked_at);

-- Snapshots indexes
CREATE INDEX idx_snapshots_issue ON issue_snapshots(issue_id);
CREATE INDEX idx_snapshots_level ON issue_snapshots(compaction_level);
CREATE INDEX idx_comp_snap_issue_level_created ON compaction_snapshots(issue_id, compaction_level, created_at DESC);

-- Repository mtimes indexes
CREATE INDEX idx_repo_mtimes_checked ON repo_mtimes(last_checked);

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Ready issues: open issues that are not blocked by any dependencies
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

-- Blocked issues: issues that are blocked with count of blockers
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

-- ============================================================================
-- USEFUL QUERIES
-- ============================================================================

-- Find ready issues (open and not blocked)
-- SELECT * FROM ready_issues ORDER BY priority ASC;

-- Find blocked issues with blocker count
-- SELECT * FROM blocked_issues ORDER BY blocked_by_count DESC;

-- Find issues by label
-- SELECT DISTINCT i.* FROM issues i
-- JOIN labels l ON i.id = l.issue_id
-- WHERE l.label = 'phase-6'
-- ORDER BY i.priority ASC;

-- Find dependency chain (recursive)
-- WITH RECURSIVE deps AS (
--   SELECT issue_id, depends_on_id, 1 as depth
--   FROM dependencies
--   WHERE issue_id = 'mcb-123'
--   UNION ALL
--   SELECT d.issue_id, d.depends_on_id, deps.depth + 1
--   FROM dependencies d
--   JOIN deps ON d.issue_id = deps.depends_on_id
--   WHERE deps.depth < 10
-- )
-- SELECT DISTINCT depends_on_id FROM deps;

-- Find overdue issues
-- SELECT * FROM issues
-- WHERE due_at < CURRENT_TIMESTAMP
--   AND status != 'closed'
-- ORDER BY due_at ASC;

-- Find issues by assignee and status
-- SELECT * FROM issues
-- WHERE assignee = 'user@example.com'
--   AND status IN ('open', 'in_progress')
-- ORDER BY priority ASC, created_at DESC;

-- Find recently updated issues
-- SELECT * FROM issues
-- WHERE updated_at > datetime('now', '-7 days')
-- ORDER BY updated_at DESC;

-- Count issues by status
-- SELECT status, COUNT(*) as count
-- FROM issues
-- GROUP BY status
-- ORDER BY count DESC;

-- Find issues with multiple labels
-- SELECT i.id, i.title, COUNT(l.label) as label_count
-- FROM issues i
-- LEFT JOIN labels l ON i.id = l.issue_id
-- GROUP BY i.id
-- HAVING label_count > 2
-- ORDER BY label_count DESC;
