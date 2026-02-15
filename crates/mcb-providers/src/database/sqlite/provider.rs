//! `SQLite` Database Provider
//!
//! # Overview
//! The `SqliteDatabaseProvider` acts as the factory and lifecycle manager for `SQLite` connections.
//! It is responsible for initializing the database file, applying the schema (DDL), and
//! creating repository instances backed by a shared `DatabaseExecutor`.
//!
//! # Responsibilities
//! - **Connection Management**: Pooling and configuring `SQLite` connections (WAL mode, etc.).
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

/// `SQLite` database provider implementation.
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
#[tracing::instrument(skip_all)]
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let (repo, _) = create_memory_repository_with_executor(path).await?;
    Ok(repo)
}

/// Create a file-backed agent repository sharing the same `SQLite` project schema database.
#[tracing::instrument(skip_all)]
pub async fn create_agent_repository(path: PathBuf) -> Result<Arc<dyn AgentRepository>> {
    let (_, executor) = create_memory_repository_with_executor(path).await?;
    Ok(create_agent_repository_from_executor(executor))
}

/// Create file-backed memory repository and executor (same DB) for use with agent repository.
#[tracing::instrument(skip_all)]
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
#[tracing::instrument(skip_all)]
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
    tracing::info!(path = %path.display(), "connecting to SQLite database");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::memory_with_source("create db directory", e))?;
    }
    let db_url = format!("sqlite:{}?mode=rwc", path.display());

    match try_connect_and_init(&path, &db_url).await {
        Ok(pool) => Ok(pool),
        Err(first_err) if path.exists() => {
            tracing::warn!(
                error = %first_err,
                path = %path.display(),
                "Database initialization failed on existing file, backing up and recreating"
            );
            backup_and_remove(&path)?;

            let fresh_pool = sqlx::SqlitePool::connect(&db_url)
                .await
                .map_err(|e| Error::memory_with_source("reconnect SQLite after backup", e))?;
            configure_pragmas(&fresh_pool).await?;
            apply_schema(&fresh_pool).await?;
            tracing::info!(
                path = %path.display(),
                "memory database recreated (old data backed up)"
            );
            Ok(fresh_pool)
        }
        Err(first_err) => {
            tracing::error!(
                error = %first_err,
                path = %path.display(),
                "Database initialization failed on NEW file (path.exists()=false)"
            );
            Err(first_err)
        }
    }
}

async fn try_connect_and_init(path: &std::path::Path, db_url: &str) -> Result<sqlx::SqlitePool> {
    use mcb_domain::error::Error;

    let pool = sqlx::SqlitePool::connect(db_url)
        .await
        .map_err(|e| Error::memory_with_source("connect SQLite", e))?;

    if let Err(e) = configure_pragmas(&pool).await {
        pool.close().await;
        return Err(e);
    }

    match apply_schema(&pool).await {
        Ok(()) => {
            tracing::info!(path = %path.display(), "memory database initialized");
            Ok(pool)
        }
        Err(schema_err) => {
            pool.close().await;
            Err(schema_err)
        }
    }
}

async fn configure_pragmas(pool: &sqlx::SqlitePool) -> Result<()> {
    use mcb_domain::error::Error;

    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(pool)
        .await
        .map_err(|e| Error::memory_with_source("enable WAL mode", e))?;

    sqlx::query("PRAGMA synchronous = NORMAL;")
        .execute(pool)
        .await
        .map_err(|e| Error::memory_with_source("set synchronous mode", e))?;

    Ok(())
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

    migrate_and_verify_schema(pool).await?;
    Ok(())
}

/// Migrate missing columns via `ALTER TABLE ADD COLUMN`, then verify all
/// tables match the expected schema.  This prevents data loss on schema
/// evolution — without it, existing databases would be backed-up and
/// recreated from scratch whenever a new nullable column is introduced.
async fn migrate_and_verify_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    use mcb_domain::error::Error;

    let schema = ProjectSchema::definition();
    for table_def in &schema.tables {
        let present = get_existing_columns(pool, &table_def.name).await?;
        if present.is_empty() {
            return Err(Error::memory(format!(
                "legacy/incompatible schema detected: missing table '{}'",
                table_def.name
            )));
        }

        for col in &table_def.columns {
            if present.contains(&col.name) {
                continue;
            }
            // Primary-key or NOT-NULL-without-default columns cannot be added
            // via ALTER TABLE in SQLite — those indicate a fundamentally
            // incompatible schema that requires a full recreate.
            if col.primary_key || col.not_null {
                return Err(Error::memory(format!(
                    "legacy/incompatible schema detected: table '{}' missing non-nullable column '{}'",
                    table_def.name, col.name
                )));
            }
            let alter_sql = super::ddl::alter_table_add_column_sqlite(&table_def.name, col);
            tracing::info!(
                table = %table_def.name,
                column = %col.name,
                "Migrating schema: adding missing column"
            );
            sqlx::query(&alter_sql).execute(pool).await.map_err(|e| {
                Error::memory_with_source(
                    format!(
                        "migrate schema: ALTER TABLE {} ADD COLUMN {}",
                        table_def.name, col.name
                    ),
                    e,
                )
            })?;
        }
    }

    Ok(())
}

/// Read the set of column names currently present in a table.
async fn get_existing_columns(pool: &sqlx::SqlitePool, table: &str) -> Result<HashSet<String>> {
    use mcb_domain::error::Error;
    use sqlx::Row;

    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::memory_with_source(format!("read schema for table {table}"), e))?;

    Ok(rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect())
}
