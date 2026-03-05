//! Database migration ports.

use std::sync::Arc;

/// Domain port for database migration providers.
///
/// Implementations supply ordered migration objects as type-erased
/// `Box<dyn Any + Send>`.  The concrete migration trait (e.g.
/// `sea_orm_migration::MigrationTrait`) lives in the infrastructure layer;
/// the domain remains free of ORM dependencies.
#[async_trait::async_trait]
pub trait MigrationProvider: Send + Sync {
    /// Returns the ordered list of migrations as type-erased boxes.
    ///
    /// Each element is expected to be a `Box<dyn MigrationTrait>` from the
    /// ORM framework used by the implementing provider.
    fn migrations(&self) -> Vec<Box<dyn std::any::Any + Send>>;

    /// Apply all pending migrations (up) to the given database connection.
    ///
    /// The `db` parameter is a type-erased database connection
    /// (e.g. `DatabaseConnection` from `SeaORM`).
    ///
    /// `steps` limits how many pending migrations to apply; `None` applies all.
    async fn migrate_up(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        steps: Option<u32>,
    ) -> crate::error::Result<()>;

    /// Rollback applied migrations on the given database connection.
    ///
    /// `steps` limits how many migrations to rollback; `None` rolls back all.
    async fn migrate_down(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        steps: Option<u32>,
    ) -> crate::error::Result<()>;
}

/// Shared migration provider for dependency injection.
pub type SharedMigrationProvider = Arc<dyn MigrationProvider>;
