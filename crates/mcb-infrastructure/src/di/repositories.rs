#![allow(clippy::missing_errors_doc, missing_docs)]

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryRepository;
use mcb_domain::registry::database::resolve_database_repositories;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use mcb_providers::migration::Migrator; // CA-EXCEPTION: SeaORM migration requirement

pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let db = connect_sqlite_with_migrations(&path).await?;
    let repos = resolve_database_repositories("seaorm", Box::new(db), "default".to_owned())
        .map_err(mcb_domain::error::Error::configuration)?;
    Ok(repos.memory)
}

pub async fn create_memory_repository_with_db(
    path: PathBuf,
) -> Result<(Arc<dyn MemoryRepository>, Arc<DatabaseConnection>)> {
    let db = connect_sqlite_with_migrations(&path).await?;
    let db = Arc::new(db);
    let repos =
        resolve_database_repositories("seaorm", Box::new((*db).clone()), "default".to_owned())
            .map_err(mcb_domain::error::Error::configuration)?;
    let repo = repos.memory;
    Ok((repo, db))
}

pub async fn connect_sqlite_with_migrations(path: &std::path::Path) -> Result<DatabaseConnection> {
    let db = connect_sqlite(path).await?;
    Migrator::up(&db, None)
        .await
        .map_err(|e| mcb_domain::error::Error::internal(format!("Migration: {e}")))?;
    Ok(db)
}

pub async fn connect_sqlite(path: &std::path::Path) -> Result<DatabaseConnection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to create database directory: {e}"))
        })?;
    }
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let mut opts = ConnectOptions::new(url);
    opts.max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);
    Database::connect(opts)
        .await
        .map_err(|e| mcb_domain::error::Error::internal(format!("Database connect: {e}")))
}
