//! SQLite backend for the memory and full project schema.
//!
//! Implements [`MemorySchemaDdlGenerator`] (memory subset) and [`SchemaDdlGenerator`]
//! (full project: collections, observations, session_summaries, file_hashes) for SQLite.
//! Provides [`SqliteExecutor`] (port [`DatabaseExecutor`]), [`SqliteMemoryRepository`]
//! (port [`MemoryRepository`]), and factory functions for DI.

mod agent_repository;
mod agent_repository;
mod ddl;
mod executor;
mod memory_repository;
mod row_convert;

pub use agent_repository::SqliteAgentRepository;
pub use agent_repository::SqliteAgentRepository;
pub use ddl::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};
pub use executor::SqliteExecutor;
pub use memory_repository::SqliteMemoryRepository;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, DatabaseProvider};
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::schema::{ProjectSchema, SchemaDdlGenerator};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SqliteDatabaseProvider;

#[async_trait]
impl DatabaseProvider for SqliteDatabaseProvider {
    async fn connect(&self, path: &Path) -> Result<Arc<dyn DatabaseExecutor>> {
        let pool = connect_and_init(path.to_path_buf()).await?;
        Ok(Arc::new(SqliteExecutor::new(pool)))
    }

    async fn connect_in_memory(&self) -> Result<Arc<dyn DatabaseExecutor>> {
        let pool = connect_in_memory_and_init().await?;
        Ok(Arc::new(SqliteExecutor::new(pool)))
    }
}

/// Create a file-backed memory repository: connect, apply [`ProjectSchema`] DDL, return repository.
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let pool = connect_and_init(path).await?;
    let executor = Arc::new(SqliteExecutor::new(pool));
    Ok(Arc::new(SqliteMemoryRepository::new(executor)))
}

pub async fn create_memory_repository_in_memory() -> Result<Arc<dyn MemoryRepository>> {
    let pool = connect_in_memory_and_init().await?;
    let executor = Arc::new(SqliteExecutor::new(pool));
    Ok(Arc::new(SqliteMemoryRepository::new(executor)))
}

pub async fn create_memory_repository_in_memory_with_executor()
-> Result<(Arc<dyn MemoryRepository>, SqliteExecutor)> {
    let pool = connect_in_memory_and_init().await?;
    let executor = SqliteExecutor::new(pool.clone());
    let executor_arc = Arc::new(SqliteExecutor::new(pool));
    Ok((
        Arc::new(SqliteMemoryRepository::new(executor_arc)),
        executor,
    ))
}

async fn connect_and_init(path: PathBuf) -> Result<sqlx::SqlitePool> {
    use mcb_domain::error::Error;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::memory_with_source("create db directory", e))?;
    }
    let db_url = format!("sqlite:{}?mode=rwc", path.display());
    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .map_err(|e| Error::memory_with_source("connect SQLite", e))?;
    apply_schema(&pool).await?;
    tracing::info!("Memory database initialized at {}", path.display());
    Ok(pool)
}

async fn connect_in_memory_and_init() -> Result<sqlx::SqlitePool> {
    use mcb_domain::error::Error;
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|e| Error::memory_with_source("connect in-memory SQLite", e))?;
    apply_schema(&pool).await?;
    tracing::debug!("In-memory memory database initialized");
    Ok(pool)
}

async fn apply_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    use mcb_domain::error::Error;
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl = generator.generate_ddl(&schema);
    for sql in ddl {
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| Error::memory_with_source("apply DDL", e))?;
    }
    Ok(())
}
