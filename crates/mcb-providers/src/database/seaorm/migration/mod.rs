//! Deterministic schema migrations for the MCB database.
use sea_orm_migration::prelude::*;

mod m20260301_000001_initial_schema;
mod m20260301_000002_workflow_schema;

/// Registers all migrations in application order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260301_000001_initial_schema::Migration),
            Box::new(m20260301_000002_workflow_schema::Migration),
        ]
    }
}
