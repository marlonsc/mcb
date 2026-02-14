//! SQLite Database Provider
//!
//! # Overview
//! The `SqliteDatabaseProvider` acts as the factory and lifecycle manager for SQLite connections.
//! It is responsible for initializing the database file, applying the schema (DDL), and
//! creating repository instances backed by a shared `DatabaseExecutor`.
//!
//! # Responsibilities
//! - **Connection Management**: Pooling and configuring SQLite connections (WAL mode, etc.).
//! - **Schema Migration**: Applying DDL at startup and verifying schema integrity.
//! - **Factory Methods**: Creating `MemoryRepository`, `AgentRepository`, etc. from a path or executor.
//! - **Recovery**: Automatically backing up and recreating Corrupt/Incompatible databases.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, DatabaseProvider};
use mcb_domain::ports::repositories::{AgentRepository, MemoryRepository, ProjectRepository};
use mcb_domain::schema::{ProjectSchema, SchemaDdlGenerator};

use super::{
    SqliteAgentRepository, SqliteExecutor, SqliteMemoryRepository, SqliteProjectRepository,
    SqliteSchemaDdlGenerator,
};
use mcb_domain::registry::database::{
    DATABASE_PROVIDERS, DatabaseProviderConfig, DatabaseProviderEntry,
};

/// SQLite database provider implementation.
///
/// Implements `DatabaseProvider` port to serve as the entry point for infrastructure composition.
pub struct SqliteDatabaseProvider;

/// Provider factory function
fn create_sqlite_database_provider(
    _config: &DatabaseProviderConfig,
) -> std::result::Result<Arc<dyn DatabaseProvider>, String> {
    Ok(Arc::new(SqliteDatabaseProvider))
}

#[linkme::distributed_slice(DATABASE_PROVIDERS)]
static SQLITE_DATABASE_PROVIDER: DatabaseProviderEntry = DatabaseProviderEntry {
    name: "sqlite",
    description: "SQLite database backend",
    factory: create_sqlite_database_provider,
};

#[async_trait]
impl DatabaseProvider for SqliteDatabaseProvider {
    async fn connect(&self, path: &Path) -> Result<Arc<dyn DatabaseExecutor>> {
        let pool = connect_and_init(path.to_path_buf()).await?;
        Ok(Arc::new(SqliteExecutor::new(pool)))
    }
}

/// Create a file-backed memory repository: connect, apply [`ProjectSchema`] DDL, return repository.
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let (repo, _) = create_memory_repository_with_executor(path).await?;
    Ok(repo)
}

/// Create a file-backed agent repository sharing the same SQLite project schema database.
pub async fn create_agent_repository(path: PathBuf) -> Result<Arc<dyn AgentRepository>> {
    let (_, executor) = create_memory_repository_with_executor(path).await?;
    Ok(create_agent_repository_from_executor(executor))
}

/// Create file-backed memory repository and executor (same DB) for use with agent repository.
pub async fn create_memory_repository_with_executor(
    path: PathBuf,
) -> Result<(Arc<dyn MemoryRepository>, Arc<dyn DatabaseExecutor>)> {
    let pool = connect_and_init(path).await?;
    let executor: Arc<dyn DatabaseExecutor> = Arc::new(SqliteExecutor::new(pool));
    let memory_repository = Arc::new(SqliteMemoryRepository::new(Arc::clone(&executor)));
    Ok((memory_repository, executor))
}

/// Create an agent repository backed by the provided database executor.
pub fn create_agent_repository_from_executor(
    executor: Arc<dyn DatabaseExecutor>,
) -> Arc<dyn AgentRepository> {
    Arc::new(SqliteAgentRepository::new(executor))
}

/// Create a file-backed project repository: connect, apply [`ProjectSchema`] DDL, return repository.
pub async fn create_project_repository(path: PathBuf) -> Result<Arc<dyn ProjectRepository>> {
    let pool = connect_and_init(path).await?;
    let executor: Arc<dyn DatabaseExecutor> = Arc::new(SqliteExecutor::new(pool));
    Ok(Arc::new(SqliteProjectRepository::new(executor)))
}

/// Create a project repository backed by the provided database executor.
pub fn create_project_repository_from_executor(
    executor: Arc<dyn DatabaseExecutor>,
) -> Arc<dyn ProjectRepository> {
    Arc::new(SqliteProjectRepository::new(executor))
}

async fn connect_and_init(path: PathBuf) -> Result<sqlx::SqlitePool> {
    use mcb_domain::error::Error;
    tracing::info!("Connecting to SQLite database at: {}", path.display());

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::memory_with_source("create db directory", e))?;
    }
    let db_url = format!("sqlite:{}?mode=rwc", path.display());
    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .map_err(|e| Error::memory_with_source("connect SQLite", e))?;

    // Enable WAL mode for better concurrency
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await
        .map_err(|e| Error::memory_with_source("enable WAL mode", e))?;

    sqlx::query("PRAGMA synchronous = NORMAL;")
        .execute(&pool)
        .await
        .map_err(|e| Error::memory_with_source("set synchronous mode", e))?;

    match apply_schema(&pool).await {
        Ok(()) => {
            tracing::info!("Memory database initialized at {}", path.display());
            Ok(pool)
        }
        Err(first_err) => {
            // Check existence explicitly to debug potential issues
            let exists = path.exists();
            if exists {
                tracing::warn!(
                    error = %first_err,
                    path = %path.display(),
                    "DDL failed on existing database, backing up and recreating"
                );
                pool.close().await;
                // TODO(architecture): Decouple destructive recovery policy.
                // The provider should likely report a SchemaMismatch error, and a higher-level
                // bootstrap service should decide whether to backup and recreate.
                backup_and_remove(&path)?;

                let fresh_pool = sqlx::SqlitePool::connect(&db_url)
                    .await
                    .map_err(|e| Error::memory_with_source("reconnect SQLite after backup", e))?;
                apply_schema(&fresh_pool).await?;
                tracing::info!(
                    "Memory database recreated at {} (old data backed up)",
                    path.display()
                );
                Ok(fresh_pool)
            } else {
                tracing::error!(
                    error = %first_err,
                    path = %path.display(),
                    "DDL failed on NEW database (path.exists()=false). This indicates a serious issue."
                );
                Err(first_err)
            }
        }
    }
}

fn backup_and_remove(path: &std::path::Path) -> Result<()> {
    use mcb_domain::error::Error;
    let stamp = mcb_domain::utils::time::epoch_secs_u64()
        .map_err(|e| mcb_domain::error::Error::memory_with_source("read system clock", e))?;
    let backup = path.with_extension(format!("db.bak.{stamp}"));
    std::fs::rename(path, &backup)
        .map_err(|e| Error::memory_with_source("backup old database", e))?;
    for ext in &["db-wal", "db-shm"] {
        let wal = path.with_extension(ext);
        if wal.exists() {
            let _ = std::fs::remove_file(&wal);
        }
    }
    tracing::info!(backup = %backup.display(), "Old database backed up");
    Ok(())
}

async fn apply_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    use mcb_domain::error::Error;
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl = generator.generate_ddl(&schema);
    let ddl_len = ddl.len();

    // Acquire a single connection for all DDL operations to ensure visibility
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| Error::memory_with_source("acquire DDL connection", e))?;

    for (index, sql) in ddl.into_iter().enumerate() {
        let stmt_preview = sql
            .lines()
            .next()
            .unwrap_or(sql.as_str())
            .chars()
            .take(120)
            .collect::<String>();
        sqlx::query(&sql).execute(&mut *conn).await.map_err(|e| {
            Error::memory_with_source(
                format!(
                    "apply DDL statement {}/{} ({})",
                    index + 1,
                    ddl_len,
                    stmt_preview
                ),
                e,
            )
        })?;
    }

    verify_project_schema(pool).await?;
    Ok(())
}

async fn verify_project_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    // TODO(architecture): Derive schema verification from domain definition to avoid drift.
    // Manual column listing here is error-prone and duplicates the schema source of truth.
    verify_table_columns(pool, "projects", &["id", "org_id", "name", "path"]).await?;
    verify_table_columns(
        pool,
        "collections",
        &["id", "project_id", "name", "vector_name"],
    )
    .await?;
    verify_table_columns(
        pool,
        "session_summaries",
        &["id", "project_id", "session_id", "origin_context"],
    )
    .await?;
    verify_table_columns(
        pool,
        "observations",
        &[
            "id",
            "project_id",
            "metadata",
            mcb_domain::schema::memory::COL_OBSERVATION_TYPE,
        ],
    )
    .await?;
    verify_table_columns(
        pool,
        "agent_sessions",
        &[
            "id",
            "session_summary_id",
            "parent_session_id",
            "project_id",
            "worktree_id",
            "model",
        ],
    )
    .await?;
    verify_table_columns(
        pool,
        "worktrees",
        &["id", "repository_id", "path", "assigned_agent_id"],
    )
    .await?;

    Ok(())
}

async fn verify_table_columns(
    pool: &sqlx::SqlitePool,
    table: &str,
    required: &[&str],
) -> Result<()> {
    use mcb_domain::error::Error;
    use sqlx::Row;

    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::memory_with_source(format!("read schema for table {table}"), e))?;

    if rows.is_empty() {
        return Err(Error::memory(format!(
            "legacy/incompatible schema detected: missing table '{table}'"
        )));
    }

    let present: HashSet<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();

    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|column| !present.contains(*column))
        .collect();

    if !missing.is_empty() {
        return Err(Error::memory(format!(
            "legacy/incompatible schema detected: table '{table}' missing required columns [{}]",
            missing.join(", ")
        )));
    }

    Ok(())
}
