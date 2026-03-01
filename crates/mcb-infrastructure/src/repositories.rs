//! `SQLite` database connection and migration utilities.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryRepository;
use mcb_domain::registry::database::resolve_database_repositories;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use mcb_providers::migration::Migrator; // CA-EXCEPTION: SeaORM migration requirement

/// Creates a memory repository backed by a `SQLite` database at the given path.
///
/// # Errors
///
/// Returns an error if the database connection, migration, or repository
/// resolution fails.
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let db = connect_sqlite_with_migrations(&path).await?;
    let repos = resolve_database_repositories("seaorm", Box::new(db), "default".to_owned())?;
    Ok(repos.memory)
}

/// Creates a memory repository and returns it alongside the raw database connection.
///
/// # Errors
///
/// Returns an error if the database connection, migration, or repository
/// resolution fails.
pub async fn create_memory_repository_with_db(
    path: PathBuf,
) -> Result<(Arc<dyn MemoryRepository>, Arc<DatabaseConnection>)> {
    let db = connect_sqlite_with_migrations(&path).await?;
    let db = Arc::new(db);
    let repos =
        resolve_database_repositories("seaorm", Box::new((*db).clone()), "default".to_owned())?;
    let repo = repos.memory;
    Ok((repo, db))
}

/// Connects to a `SQLite` database and runs pending migrations.
///
/// If the initial connection or migration fails, attempts automatic recovery
/// by backing up the corrupted file and creating a fresh database.
///
/// # Errors
///
/// Returns an error if both the primary connection and recovery fail.
pub async fn connect_sqlite_with_migrations(path: &std::path::Path) -> Result<DatabaseConnection> {
    // Try to connect and run migrations
    match connect_sqlite(path).await {
        Ok(db) => {
            // Try to run migrations on the connected database
            match Migrator::up(&db, None).await {
                Ok(_) => Ok(db),
                Err(e) => {
                    // Migration failed - likely corrupted DB
                    mcb_domain::info!(
                        "db_recovery",
                        "Migration failed, attempting recovery",
                        &format!("error: {e}")
                    );
                    recover_database(path).await
                }
            }
        }
        Err(e) => {
            // Connection failed - likely corrupted DB
            mcb_domain::info!(
                "db_recovery",
                "Connection failed, attempting recovery",
                &format!("error: {e}")
            );
            recover_database(path).await
        }
    }
}

/// Opens a `SQLite` connection at the given path, creating the file and parent
/// directories if they do not exist.
///
/// # Errors
///
/// Returns an error if directory creation or the database connection fails.
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

/// Recover a corrupted database by backing it up and creating a fresh one.
async fn recover_database(path: &std::path::Path) -> Result<DatabaseConnection> {
    // Backup the corrupted database
    if path.exists() {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_path = format!("{}.bak.{}", path.display(), timestamp);
        fs::copy(path, &backup_path).map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to backup corrupted database: {e}"))
        })?;
        mcb_domain::info!(
            "db_recovery",
            "backing up and recreating",
            &format!("backed up to {backup_path}")
        );
        fs::remove_file(path).map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to remove corrupted database: {e}"))
        })?;
    }

    // Create a fresh database
    let db = connect_sqlite(path).await?;
    mcb_domain::info!(
        "db_recovery",
        "Memory database recreated",
        &"fresh database created"
    );

    // Run migrations on the fresh database
    Migrator::up(&db, None).await.map_err(|e| {
        mcb_domain::error::Error::internal(format!(
            "Failed to run migrations on recovered database: {e}"
        ))
    })?;

    Ok(db)
}
