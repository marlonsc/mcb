//! SeaORM migration provider implementation.

use std::any::Any;
use std::sync::Arc;

use mcb_domain::ports::MigrationProvider;
use sea_orm_migration::prelude::*;

use super::Migrator;

/// `SeaORM` migration provider implementing the domain `MigrationProvider` port.
struct SeaOrmMigrationProvider;

#[async_trait::async_trait]
impl MigrationProvider for SeaOrmMigrationProvider {
    fn migrations(&self) -> Vec<Box<dyn Any + Send>> {
        super::mcb_migrations()
            .into_iter()
            .map(|m| Box::new(m) as Box<dyn Any + Send>)
            .collect()
    }

    async fn migrate_up(
        &self,
        db: Box<dyn Any + Send + Sync>,
        steps: Option<u32>,
    ) -> mcb_domain::error::Result<()> {
        let conn = db.downcast::<sea_orm::DatabaseConnection>().map_err(|_| {
            mcb_domain::error::Error::configuration("migrate_up: expected DatabaseConnection")
        })?;
        Migrator::up(&*conn, steps)
            .await
            .map_err(|e| mcb_domain::error::Error::configuration(e.to_string()))
    }

    async fn migrate_down(
        &self,
        db: Box<dyn Any + Send + Sync>,
        steps: Option<u32>,
    ) -> mcb_domain::error::Result<()> {
        let conn = db.downcast::<sea_orm::DatabaseConnection>().map_err(|_| {
            mcb_domain::error::Error::configuration("migrate_down: expected DatabaseConnection")
        })?;
        Migrator::down(&*conn, steps)
            .await
            .map_err(|e| mcb_domain::error::Error::configuration(e.to_string()))
    }
}

/// Build the `SeaORM` migration provider.
fn build_seaorm_migration_provider() -> Arc<dyn MigrationProvider> {
    Arc::new(SeaOrmMigrationProvider)
}

mcb_domain::register_migration_provider!(
    mcb_utils::constants::DEFAULT_DATABASE_PROVIDER,
    "SeaORM schema migrations (SQLite, PostgreSQL, MySQL)",
    build_seaorm_migration_provider,
);
