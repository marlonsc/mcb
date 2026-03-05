//! Unified SeaORM entity repository.

use std::sync::Arc;

use sea_orm::DatabaseConnection;

/// Unified SeaORM-backed entity repository implementing all entity CRUD traits.
pub struct SeaOrmEntityRepository {
    pub(super) db: Arc<DatabaseConnection>,
}

impl SeaOrmEntityRepository {
    /// Creates a new entity repository backed by the given database connection.
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Returns a reference to the underlying database connection.
    #[must_use]
    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }
}
