# ADR-035: Context Scout — Project State Discovery

## Status

**Proposed** — 2026-02-05

-   **Deciders:** Project team
-   **Depends on:** [ADR-034](./034-workflow-core-fsm.md) (Workflow Core FSM)
-   **Related:** [ADR-029](./029-hexagonal-architecture-dill.md) (Hexagonal DI), [ADR-023](./023-provider-registration-linkme.md) (linkme), [ADR-025](./025-figment-configuration.md) (Figment)
-   **Series:** [ADR-034](./034-workflow-core-fsm.md) → **ADR-035** → [ADR-036](./036-enforcement-policies.md) → [ADR-037](./037-workflow-orchestrator.md)

## Context

ADR-034 defines the workflow FSM and its persistence layer. The FSM transitions between states (Initializing → Ready → Planning → Executing → ...), but the transition from `Initializing` to `Ready` requires a **context snapshot** — a typed view of the current project state.

Today, context discovery is scattered:

| Source | Current Mechanism | Problem |
|--------|-------------------|---------|
| Git status | Shell `git status --porcelain` | Parsed ad-hoc, not typed, not cached |
| Branch info | Shell `git branch --show-current` | Same |
| Issue tracker | `bd ready`, `bd list` (Beads CLI) | External process, JSON parsing, slow |
| Project phases | `.planning/STATE.md` (GSD) | Markdown, no schema, no search |
| Stash/commits | Shell commands | No integration with MCB |

**This ADR** defines a typed `ProjectContext` entity and a `ContextScoutProvider` port that discovers and caches project state using `git2` (already in MCB's dependency tree) and direct SQLite queries (for issues/phases stored by the workflow engine).

### Requirements

-   Discover git state without shelling out (use `git2` library directly)
-   Provide a single `ProjectContext` struct with all relevant state
-   Cache snapshots with configurable TTL to avoid re-scanning on every operation
-   Support incremental discovery (git-only, tracker-only, or full)
-   Expose as a provider trait consumed by ADR-036 (policies) and ADR-037 (orchestrator)

## Decision

### 1. Use `git2` (Not `gix`)

MCB already depends on `git2` for code indexing. Using `gix` (gitoxide) would add a second git library to the dependency tree for marginal benefit.

| Aspect | git2 | gix |
|--------|------|-----|
| Already in MCB deps | Yes | No |
| API maturity | Stable, well-documented | Newer, more verbose |
| Performance | Good for repo sizes MCB targets | 500-1000x faster for huge monorepos |
| Async | No (use `spawn_blocking`) | Partial |

`git2` operations are blocking (libgit2 FFI). All calls are wrapped in `tokio::task::spawn_blocking()` to avoid blocking the Tokio runtime.

### 2. Domain Entities

```rust
// mcb-domain/src/entities/context.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete snapshot of project state at a point in time.
/// Produced by ContextScoutProvider, consumed by PolicyGuardProvider (ADR-036)
/// and WorkflowService (ADR-037).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Unique snapshot identifier.
    pub id: String,
    /// Project root directory.
    pub project_root: PathBuf,
    /// Git repository state.
    pub git: GitContext,
    /// Issue tracker state (from workflow SQLite).
    pub tracker: TrackerContext,
    /// Project configuration.
    pub config: ProjectConfig,
    /// When this snapshot was captured.
    pub discovered_at: DateTime<Utc>,
}

/// Git repository state via `git2`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    /// Current branch name (e.g., "release/v0.3.0").
    pub branch: String,
    /// Number of staged files (index changes).
    pub staged_files: u32,
    /// Number of unstaged modified files (working tree changes).
    pub unstaged_files: u32,
    /// Number of untracked files.
    pub untracked_files: u32,
    /// Number of conflicted files.
    pub conflicted_files: u32,
    /// Number of stash entries.
    pub stash_count: u32,
    /// Most recent commits (newest first).
    pub recent_commits: Vec<CommitSummary>,
    /// True if worktree is completely clean (no staged, unstaged, or untracked).
    pub is_clean: bool,
    /// Repository state (Clean, Merge, Rebase, etc.).
    pub repo_state: String,
}

/// Summary of a single commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

/// Issue tracker state (reads from workflow SQLite tables defined in ADR-034).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerContext {
    /// Total open issues.
    pub total_open: u32,
    /// Issues in progress.
    pub in_progress: Vec<IssueSummary>,
    /// Issues ready to start (no blockers).
    pub ready: Vec<IssueSummary>,
    /// Blocked issues.
    pub blocked: Vec<IssueSummary>,
    /// Current active phase, if any.
    pub current_phase: Option<PhaseSummary>,
    /// Overall progress percentage.
    pub progress_percent: f64,
}

/// Summary of a single issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    pub id: String,
    pub title: String,
    pub issue_type: String,
    pub priority: u8,
    pub status: String,
}

/// Summary of a project phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSummary {
    pub id: String,
    pub phase_number: u32,
    pub title: String,
    pub status: String,
    pub progress: f64,
}

/// Project-level configuration relevant to the workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project identifier (from mcb.toml).
    pub project_id: String,
    /// Project name.
    pub name: String,
    /// Version being developed.
    pub version: String,
}
```

### 3. Port Trait

```rust
// mcb-domain/src/ports/providers/context_scout.rs

use crate::entities::context::{GitContext, ProjectContext, TrackerContext};
use crate::errors::WorkflowError;
use std::path::Path;

/// Port for project state discovery.
///
/// Discovers git status, issue tracker state, and project configuration.
/// Snapshots are cached with configurable TTL.
/// Consumed by PolicyGuardProvider (ADR-036) and WorkflowService (ADR-037).
#[async_trait::async_trait]
pub trait ContextScoutProvider: Send + Sync {
    /// Full project context snapshot. Cached with TTL.
    async fn discover(&self, project_root: &Path) -> Result<ProjectContext, WorkflowError>;

    /// Git-only state (faster, no tracker query). Cached separately.
    async fn git_status(&self, project_root: &Path) -> Result<GitContext, WorkflowError>;

    /// Tracker-only state (issues, phases). Cached separately.
    async fn tracker_state(&self, project_id: &str) -> Result<TrackerContext, WorkflowError>;

    /// Invalidate all cached snapshots for a project.
    async fn invalidate_cache(&self, project_root: &Path) -> Result<(), WorkflowError>;
}
```

### 4. Git Discovery Implementation

```rust
// mcb-providers/src/context/git_discovery.rs

use git2::{Repository, StatusOptions, StatusShow};
use mcb_domain::entities::context::{CommitSummary, GitContext};
use mcb_domain::errors::WorkflowError;

/// Discover git repository state using git2.
/// MUST run inside tokio::task::spawn_blocking() — git2 is blocking FFI.
pub fn discover_git_status(
    repo_path: &std::path::Path,
    max_commits: usize,
) -> Result<GitContext, WorkflowError> {
    let repo = Repository::open(repo_path)
        .map_err(|e| WorkflowError::ContextError { message: format!("Failed to open repo: {e}") })?;

    // Branch name
    let branch = match repo.head() {
        Ok(head) => head.shorthand().unwrap_or("HEAD").to_string(),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => "(unborn)".to_string(),
        Err(e) => return Err(WorkflowError::ContextError { message: e.to_string() }),
    };

    // File status counts
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .renames_head_to_index(true);

    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

    let mut staged = 0u32;
    let mut unstaged = 0u32;
    let mut untracked = 0u32;
    let mut conflicted = 0u32;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_index_new() || s.is_index_modified() || s.is_index_deleted()
            || s.is_index_renamed() || s.is_index_typechange()
        {
            staged += 1;
        }
        if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_typechange() {
            unstaged += 1;
        }
        if s.is_wt_new() {
            untracked += 1;
        }
        if s.is_conflicted() {
            conflicted += 1;
        }
    }

    // Stash count
    let mut stash_count = 0u32;
    repo.stash_foreach(|_, _, _| {
        stash_count += 1;
        true
    })
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

    // Recent commits
    let recent_commits = discover_recent_commits(&repo, max_commits)?;

    // Repository state
    let repo_state = format!("{:?}", repo.state());

    let is_clean = staged == 0 && unstaged == 0 && untracked == 0 && conflicted == 0
        && repo.state() == git2::RepositoryState::Clean;

    Ok(GitContext {
        branch,
        staged_files: staged,
        unstaged_files: unstaged,
        untracked_files: untracked,
        conflicted_files: conflicted,
        stash_count,
        recent_commits,
        is_clean,
        repo_state,
    })
}

fn discover_recent_commits(
    repo: &Repository,
    max: usize,
) -> Result<Vec<CommitSummary>, WorkflowError> {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(Vec::new()), // No commits yet
    };

    let head_oid = head.target()
        .ok_or_else(|| WorkflowError::ContextError {
            message: "HEAD has no target".to_string(),
        })?;

    let mut revwalk = repo.revwalk()
        .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;
    revwalk.push(head_oid)
        .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;
    revwalk.simplify_first_parent()
        .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

    let mut commits = Vec::with_capacity(max);
    for oid in revwalk.take(max) {
        let oid = oid.map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;
        let commit = repo.find_commit(oid)
            .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

        let hash = oid.to_string();
        let short_hash = hash[..7.min(hash.len())].to_string();
        let message = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let time = commit.time();
        let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or_default()
            .with_timezone(&chrono::Utc);

        commits.push(CommitSummary {
            hash,
            short_hash,
            message,
            author,
            timestamp,
        });
    }

    Ok(commits)
}
```

### 5. Tracker Discovery Implementation

```rust
// mcb-providers/src/context/tracker_discovery.rs

use mcb_domain::entities::context::{IssueSummary, PhaseSummary, TrackerContext};
use mcb_domain::errors::WorkflowError;
use sqlx::SqlitePool;

/// Discover issue tracker state from workflow SQLite tables (ADR-034 schema).
pub async fn discover_tracker_state(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<TrackerContext, WorkflowError> {
    // Total open
    let total_open: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM issues WHERE status != 'closed'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

    // In-progress issues
    let in_progress = query_issues(pool, "in_progress").await?;

    // Ready issues (using ready_issues view from ADR-034 schema)
    let ready = sqlx::query_as::<_, IssueRow>(
        "SELECT id, title, type as issue_type, priority, status
         FROM ready_issues ORDER BY priority ASC LIMIT 20"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?
    .into_iter()
    .map(Into::into)
    .collect();

    // Blocked issues
    let blocked = sqlx::query_as::<_, IssueRow>(
        "SELECT i.id, i.title, i.type as issue_type, i.priority, i.status
         FROM issues i
         WHERE i.status = 'open'
           AND EXISTS (
             SELECT 1 FROM issue_dependencies d
             JOIN issues b ON d.depends_on = b.id
             WHERE d.issue_id = i.id AND b.status != 'closed'
           )
         ORDER BY i.priority ASC LIMIT 20"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?
    .into_iter()
    .map(Into::into)
    .collect();

    // Current phase
    let current_phase = sqlx::query_as::<_, PhaseRow>(
        "SELECT id, phase_number, title, status, progress
         FROM phases
         WHERE project_id = ? AND status IN ('in_progress', 'planned')
         ORDER BY phase_number ASC LIMIT 1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?
    .map(Into::into);

    // Overall progress
    let progress_percent = calculate_progress(pool, project_id).await?;

    Ok(TrackerContext {
        total_open,
        in_progress,
        ready,
        blocked,
        current_phase,
        progress_percent,
    })
}

async fn query_issues(
    pool: &SqlitePool,
    status: &str,
) -> Result<Vec<IssueSummary>, WorkflowError> {
    sqlx::query_as::<_, IssueRow>(
        "SELECT id, title, type as issue_type, priority, status
         FROM issues WHERE status = ? ORDER BY priority ASC LIMIT 20"
    )
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })
    .map(|rows| rows.into_iter().map(Into::into).collect())
}

async fn calculate_progress(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<f64, WorkflowError> {
    let row = sqlx::query(
        "SELECT COUNT(*) as total,
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as done
         FROM phases WHERE project_id = ?"
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| WorkflowError::ContextError { message: e.to_string() })?;

    let total: f64 = row.get::<i64, _>("total") as f64;
    let done: f64 = row.get::<i64, _>("done") as f64;

    if total == 0.0 { Ok(0.0) } else { Ok((done / total) * 100.0) }
}
```

### 6. Caching Strategy

Use `moka` (already in MCB for cache provider) with TTL-based invalidation:

```rust
// mcb-providers/src/context/cached_scout.rs

use moka::future::Cache;
use std::path::PathBuf;
use std::time::Duration;

pub struct CachedContextScout {
    git_cache: Cache<PathBuf, GitContext>,
    tracker_cache: Cache<String, TrackerContext>,
    full_cache: Cache<PathBuf, ProjectContext>,
    pool: SqlitePool,
    config: ContextScoutConfig,
}

impl CachedContextScout {
    pub fn new(pool: SqlitePool, config: ContextScoutConfig) -> Self {
        let ttl = Duration::from_secs(config.cache_ttl_seconds);
        Self {
            git_cache: Cache::builder()
                .max_capacity(50)
                .time_to_live(ttl)
                .time_to_idle(Duration::from_secs(config.cache_ttl_seconds / 3))
                .build(),
            tracker_cache: Cache::builder()
                .max_capacity(20)
                .time_to_live(ttl)
                .build(),
            full_cache: Cache::builder()
                .max_capacity(10)
                .time_to_live(ttl)
                .build(),
            pool,
            config,
        }
    }
}

#[async_trait::async_trait]
impl ContextScoutProvider for CachedContextScout {
    async fn discover(&self, project_root: &Path) -> Result<ProjectContext, WorkflowError> {
        let key = project_root.to_path_buf();
        if let Some(cached) = self.full_cache.get(&key).await {
            return Ok(cached);
        }

        let git = self.git_status(project_root).await?;
        let tracker = self.tracker_state(&self.config.project_id).await?;
        let config = load_project_config(project_root)?;

        let ctx = ProjectContext {
            id: uuid::Uuid::new_v4().to_string(),
            project_root: project_root.to_path_buf(),
            git,
            tracker,
            config,
            discovered_at: Utc::now(),
        };

        self.full_cache.insert(key, ctx.clone()).await;
        Ok(ctx)
    }

    async fn git_status(&self, project_root: &Path) -> Result<GitContext, WorkflowError> {
        let key = project_root.to_path_buf();
        if let Some(cached) = self.git_cache.get(&key).await {
            return Ok(cached);
        }

        let path = project_root.to_path_buf();
        let max_commits = self.config.max_recent_commits;

        // git2 is blocking FFI — run on blocking thread pool
        let git = tokio::task::spawn_blocking(move || {
            discover_git_status(&path, max_commits)
        })
        .await
        .map_err(|e| WorkflowError::ContextError { message: e.to_string() })??;

        self.git_cache.insert(key, git.clone()).await;
        Ok(git)
    }

    async fn tracker_state(&self, project_id: &str) -> Result<TrackerContext, WorkflowError> {
        let key = project_id.to_string();
        if let Some(cached) = self.tracker_cache.get(&key).await {
            return Ok(cached);
        }

        let tracker = discover_tracker_state(&self.pool, project_id).await?;
        self.tracker_cache.insert(key, tracker.clone()).await;
        Ok(tracker)
    }

    async fn invalidate_cache(&self, project_root: &Path) -> Result<(), WorkflowError> {
        let key = project_root.to_path_buf();
        self.git_cache.invalidate(&key).await;
        self.full_cache.invalidate(&key).await;
        // Tracker cache invalidated by project_id, not path
        self.tracker_cache.run_pending_tasks().await;
        Ok(())
    }
}
```

### 7. Configuration

```toml
# config/default.toml — [context] section

[context]
# Cache TTL in seconds (default 30s).
# Lower for active development, higher for CI.
cache_ttl_seconds = 30

# Maximum recent commits to include in GitContext.
max_recent_commits = 10

# Project identifier for tracker queries.
project_id = "mcb"
```

```rust
// mcb-infrastructure/src/config/context.rs

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ContextScoutConfig {
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
    #[serde(default = "default_max_commits")]
    pub max_recent_commits: usize,
    pub project_id: String,
}

fn default_cache_ttl() -> u64 { 30 }
fn default_max_commits() -> usize { 10 }
```

### 8. Provider Registration (linkme)

```rust
// mcb-application/src/registry/context.rs

use mcb_domain::ports::providers::context_scout::ContextScoutProvider;
use std::sync::Arc;

pub struct ContextProviderEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub factory: fn(&figment::Figment) -> Result<Arc<dyn ContextScoutProvider>, Box<dyn std::error::Error + Send + Sync>>,
}

#[linkme::distributed_slice]
pub static CONTEXT_PROVIDERS: [ContextProviderEntry] = [..];
```

```rust
// mcb-providers/src/context/mod.rs

#[linkme::distributed_slice(CONTEXT_PROVIDERS)]
static CACHED_SCOUT: ContextProviderEntry = ContextProviderEntry {
    name: "cached",
    description: "Cached context scout using git2 and SQLite",
    factory: cached_scout_factory,
};

fn cached_scout_factory(
    config: &figment::Figment,
) -> Result<Arc<dyn ContextScoutProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let scout_config: ContextScoutConfig = config.extract_inner("context")?;
    let workflow_config: WorkflowConfig = config.extract_inner("workflow")?;
    let pool = SqlitePool::connect_lazy(&workflow_config.database_url)?;
    Ok(Arc::new(CachedContextScout::new(pool, scout_config)))
}
```

### 9. Issue/Phase SQLite Tables

These tables are written by the orchestrator (ADR-037) and read by the Context Scout:

```sql
-- Project phases (written by orchestrator, read by scout)
CREATE TABLE IF NOT EXISTS phases (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL,
    phase_number INTEGER NOT NULL,
    title        TEXT NOT NULL,
    goal         TEXT,
    status       TEXT NOT NULL DEFAULT 'planned',
    progress     REAL DEFAULT 0.0,
    depends_on   TEXT,           -- JSON array of phase IDs
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE INDEX idx_phases_project ON phases(project_id, phase_number);

-- Issues (written by orchestrator, read by scout)
CREATE TABLE IF NOT EXISTS issues (
    id          TEXT PRIMARY KEY,
    phase_id    TEXT REFERENCES phases(id),
    title       TEXT NOT NULL,
    type        TEXT NOT NULL,       -- 'task', 'bug', 'feature'
    priority    INTEGER DEFAULT 2,   -- 0=critical, 4=backlog
    status      TEXT NOT NULL DEFAULT 'open',
    assignee    TEXT,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_phase ON issues(phase_id);

-- Issue dependencies (blocking relationships)
CREATE TABLE IF NOT EXISTS issue_dependencies (
    issue_id    TEXT NOT NULL REFERENCES issues(id),
    depends_on  TEXT NOT NULL REFERENCES issues(id),
    PRIMARY KEY (issue_id, depends_on)
);

-- Decisions log
CREATE TABLE IF NOT EXISTS decisions (
    id          TEXT PRIMARY KEY,
    phase_id    TEXT REFERENCES phases(id),
    session_id  TEXT,
    title       TEXT NOT NULL,
    rationale   TEXT,
    outcome     TEXT,
    created_at  INTEGER NOT NULL
);

-- View: issues with no unresolved blockers
CREATE VIEW IF NOT EXISTS ready_issues AS
SELECT i.* FROM issues i
WHERE i.status = 'open'
  AND NOT EXISTS (
    SELECT 1 FROM issue_dependencies d
    JOIN issues blocker ON d.depends_on = blocker.id
    WHERE d.issue_id = i.id AND blocker.status != 'closed'
  );
```

### 10. Module Locations

| Crate | Path | Content |
|-------|------|---------|
| `mcb-domain` | `src/entities/context.rs` | `ProjectContext`, `GitContext`, `TrackerContext`, `CommitSummary`, `IssueSummary`, `PhaseSummary`, `ProjectConfig` |
| `mcb-domain` | `src/ports/providers/context_scout.rs` | `ContextScoutProvider` trait |
| `mcb-application` | `src/registry/context.rs` | `CONTEXT_PROVIDERS` slice |
| `mcb-providers` | `src/context/mod.rs` | Module root + linkme registration |
| `mcb-providers` | `src/context/git_discovery.rs` | `discover_git_status()` using git2 |
| `mcb-providers` | `src/context/tracker_discovery.rs` | `discover_tracker_state()` using sqlx |
| `mcb-providers` | `src/context/cached_scout.rs` | `CachedContextScout` with moka TTL |
| `mcb-infrastructure` | `src/config/context.rs` | `ContextScoutConfig` (Figment) |

## Consequences

### Positive

-   **Zero shell dependencies**: All discovery via `git2` FFI and direct SQLite — no `git`, `bd`, or `.planning/` commands.
-   **Typed state**: `ProjectContext` with strong types eliminates String parsing errors.
-   **Performant**: Moka cache with 30s TTL. Cold: 5–20ms (git2). Warm: < 1ms.
-   **Composable**: `git_status()` and `tracker_state()` can be called independently for partial discovery.
-   **Reuses existing deps**: `git2`, `moka`, `sqlx` — all already in MCB's `Cargo.toml`.
-   **Foundation for ADR-036**: `PolicyGuardProvider` receives `ProjectContext` to evaluate policies.

### Negative

-   **git2 blocking FFI**: Requires `spawn_blocking()` wrapper, adding complexity. Cannot be fully async.
-   **Cache staleness**: 30s TTL means state could be up to 30s stale. Mitigated by manual `invalidate_cache()`.
-   **Shared SQLite pool**: Context Scout reads from the same SQLite DB the workflow engine writes. Must handle concurrent access via WAL mode.
-   **No file watcher**: Does not react to filesystem changes in real-time. Polling-based via TTL. File watcher (notify crate) deferred to future enhancement.

## Alternatives Considered

### Alternative 1: gix (gitoxide)

-   **Description:** Pure Rust git implementation. Significantly faster for large repositories.
-   **Pros:** No C FFI. 500–1000x faster on large repos. True async possible.
-   **Cons:** Not in MCB's current deps. More verbose API. Newer, less battle-tested.
-   **Rejection reason:** Adding a second git library increases binary size and maintenance burden for repositories MCB targets (small-to-medium). git2 is already proven in the indexing pipeline.

### Alternative 2: Shell-Based Discovery

-   **Description:** Shell out to `git status --porcelain`, `git stash list`, etc.
-   **Pros:** Simple. No library dependency.
-   **Cons:** Parsing fragile. Requires `git` in PATH. Cross-platform issues. Slow (process spawn per query).
-   **Rejection reason:** Violates zero-shell-deps principle. MCB must be self-contained.

### Alternative 3: No Caching

-   **Description:** Discover fresh context on every call.
-   **Pros:** Always up-to-date. Simpler implementation.
-   **Cons:** 5–20ms per call for git2 operations. Multiplied by every policy check and orchestrator call.
-   **Rejection reason:** Performance unacceptable when context is queried multiple times per workflow transition.

## Implementation Notes

### Code Changes

1.  Add `context.rs` entities to `mcb-domain/src/entities/`
2.  Add `context_scout.rs` port to `mcb-domain/src/ports/providers/`
3.  Add `CONTEXT_PROVIDERS` slice to `mcb-application/src/registry/`
4.  Add `context/` module to `mcb-providers/src/` with git discovery, tracker discovery, and cached scout
5.  Add `ContextScoutConfig` to `mcb-infrastructure/src/config/`
6.  Add `[context]` section to `config/default.toml`
7.  Create issue/phase/decision tables (if not created by ADR-034's workflow tables)

### Migration

-   New tables (`phases`, `issues`, `issue_dependencies`, `decisions`). No existing tables modified.
-   `CREATE TABLE IF NOT EXISTS` in provider initialization.

### Testing

-   Unit tests: Git discovery with `git2::Repository::init()` (temp dir).
-   Unit tests: Tracker queries with in-memory SQLite.
-   Unit tests: Cache hit/miss/invalidation with moka.
-   Integration tests: Full `ProjectContext` discovery on actual MCB repo.
-   Estimated: ~50 tests.

### Performance Targets

| Operation | Cold | Warm (Cached) |
|-----------|------|---------------|
| `git_status()` | < 20ms | < 1ms |
| `tracker_state()` | < 10ms | < 1ms |
| `discover()` (full) | < 30ms | < 1ms |
| Cache invalidation | < 1ms | N/A |

### Security

-   `git2` may expose file paths and commit messages. No credentials are stored in `ProjectContext`.
-   SQLite pool uses same security model as workflow engine (local file, no network).

## References

-   [git2 crate](https://crates.io/crates/git2) — Rust bindings for libgit2
-   [moka crate](https://crates.io/crates/moka) — Concurrent cache (already in MCB)
-   [gitui source](https://github.com/extrawurst/gitui) — Reference for git2 status patterns
-   [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md) — FSM and persistence layer (dependency)
-   [ADR-029: Hexagonal Architecture with dill](./029-hexagonal-architecture-dill.md) — DI pattern
-   [docs/design/workflow-management/SCHEMA.md](../design/workflow-management/SCHEMA.md) — Schema reference
