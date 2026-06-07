//! SeaORM database connection provider — linkme registration for
//! `DATABASE_CONNECTION_PROVIDERS`.
//!
//! Supports `sqlite` (file-based or in-memory) and `postgres` connection
//! strings.  After connecting, runs all registered migrations.

use std::any::Any;
use std::sync::Arc;

use mcb_domain::registry::database::{DatabaseBuildFuture, DatabaseProviderConfig};

/// Build a `SQLite` connection string from the provider configuration.
fn sqlite_url(config: &DatabaseProviderConfig) -> String {
    match &config.path {
        Some(path) => format!("sqlite:{}?mode=rwc", path.display()),
        None => mcb_utils::constants::SQLITE_MEMORY_DSN.to_owned(),
    }
}

/// Build a `PostgreSQL` connection string from the provider configuration.
fn postgres_url(config: &DatabaseProviderConfig) -> String {
    config
        .path
        .as_ref()
        .map_or_else(String::new, |p| p.display().to_string())
}

/// Factory: connect to the database, run migrations, return the connection.
fn build_seaorm_connection(config: &DatabaseProviderConfig) -> DatabaseBuildFuture {
    let url = match config.provider.as_str() {
        "sqlite" => sqlite_url(config),
        "postgres" | "postgresql" => postgres_url(config),
        _ => {
            let name = config.provider.clone();
            return Box::pin(async move {
                Err(mcb_domain::error::Error::configuration(format!(
                    "SeaORM connection: unsupported provider '{name}'"
                )))
            });
        }
    };

    Box::pin(async move {
        let db = sea_orm::Database::connect(&url)
            .await
            .map_err(|e| mcb_domain::error::Error::configuration(e.to_string()))?;

        // Run all pending migrations on the fresh connection.
        mcb_domain::registry::database::migrate_up(Box::new(db.clone()), None).await?;

        Ok(Arc::new(db) as Arc<dyn Any + Send + Sync>)
    })
}

mcb_domain::register_database_connection!(SQLITE_CONN, "sqlite", build_seaorm_connection);

mcb_domain::register_database_connection!(POSTGRES_CONN, "postgres", build_seaorm_connection);
