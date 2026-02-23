#![allow(clippy::missing_errors_doc, missing_docs)]

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryRepository;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::repos::SeaOrmObservationRepository;

pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let db = connect_sqlite_with_migrations(&path).await?;
    Ok(Arc::new(SeaOrmObservationRepository::new(db)))
}

pub async fn create_memory_repository_with_db(
    path: PathBuf,
) -> Result<(Arc<dyn MemoryRepository>, Arc<DatabaseConnection>)> {
    let db = connect_sqlite_with_migrations(&path).await?;
    let db = Arc::new(db);
    let repo = Arc::new(SeaOrmObservationRepository::new((*db).clone()));
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
