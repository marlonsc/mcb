//! Deterministic schema migrations for the MCB database.
use std::any::Any;
use std::sync::Arc;

use mcb_domain::ports::MigrationProvider;
use mcb_domain::registry::database::{MIGRATION_PROVIDERS, MigrationProviderEntry};
use sea_orm_migration::prelude::*;

mod m20260301_000001_initial_schema;
mod m20260301_000002_workflow_schema;

/// Returns the ordered list of migrations for the MCB database.
fn mcb_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260301_000001_initial_schema::Migration),
        Box::new(m20260301_000002_workflow_schema::Migration),
    ]
}

/// Registers all migrations in application order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        mcb_migrations()
    }
}

// ---------------------------------------------------------------------------
// CA/DI: MigrationProvider port implementation + linkme registration
// ---------------------------------------------------------------------------

/// SeaORM migration provider implementing the domain `MigrationProvider` port.
struct SeaOrmMigrationProvider;

#[async_trait::async_trait]
impl MigrationProvider for SeaOrmMigrationProvider {
    fn migrations(&self) -> Vec<Box<dyn Any + Send>> {
        mcb_migrations()
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
            .map_err(|e| mcb_domain::error::Error::configuration(&e.to_string()))
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
            .map_err(|e| mcb_domain::error::Error::configuration(&e.to_string()))
    }
}

/// Build the SeaORM migration provider.
fn build_seaorm_migration_provider() -> Arc<dyn MigrationProvider> {
    Arc::new(SeaOrmMigrationProvider)
}

/// SeaORM migration provider registration.
#[linkme::distributed_slice(MIGRATION_PROVIDERS)]
static SEAORM_MIGRATIONS: MigrationProviderEntry = MigrationProviderEntry {
    name: "seaorm",
    description: "SeaORM schema migrations (SQLite, PostgreSQL, MySQL)",
    build: build_seaorm_migration_provider,
};
