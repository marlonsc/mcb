//! DI-resolved database migrator (CA pattern via domain registry).
//!
//! This module provides [`DynamicMigrator`], a `MigratorTrait` implementation
//! that resolves migrations through the domain's [`MigrationProvider`] port.
//!
//! ## Architecture
//!
//! ```text
//! loco_app.rs
//!     └─ create_app::<McbApp, DynamicMigrator>(…)
//!            └─ DynamicMigrator::migrations()
//!                  └─ mcb_domain::registry::database::resolve_migration_provider()
//!                        └─ linkme distributed slice (MIGRATION_PROVIDERS)
//!                              └─ mcb-providers registers its entries here
//! ```

use sea_orm_migration::prelude::*;

/// Database migrator resolved through the domain CA/DI registry.
///
/// Instead of importing a concrete `Migrator` from `mcb-providers`, this type
/// resolves migrations at link-time via the
/// [`MIGRATION_PROVIDERS`](mcb_domain::registry::database::MIGRATION_PROVIDERS)
/// distributed slice and the [`MigrationProvider`](mcb_domain::ports::MigrationProvider)
/// port trait.
pub struct DynamicMigrator;

#[async_trait::async_trait]
impl MigratorTrait for DynamicMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let provider = mcb_domain::registry::database::resolve_migration_provider()
            .expect("No migration provider registered");
        provider
            .migrations()
            .into_iter()
            .filter_map(|any| {
                any.downcast::<Box<dyn MigrationTrait>>()
                    .ok()
                    .map(|boxed| *boxed)
            })
            .collect()
    }
}
