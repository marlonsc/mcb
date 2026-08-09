//! Runtime dependency readiness checks.

use std::sync::Arc;

use mcb_domain::ports::{
    EmbeddingProvider, ReadinessDependency, ReadinessProvider, ReadinessReport, VectorStoreProvider,
};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use crate::infrastructure::DynamicMigrator;

/// Readiness checker backed by the live application dependencies.
pub struct RuntimeReadiness {
    database: DatabaseConnection,
    embedding: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl RuntimeReadiness {
    /// Build a checker from the application composition root.
    #[must_use]
    pub fn new(
        database: DatabaseConnection,
        embedding: Arc<dyn EmbeddingProvider>,
        vector_store: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            database,
            embedding,
            vector_store,
        }
    }

    async fn database(&self) -> ReadinessDependency {
        let result = self.database.execute_unprepared("SELECT 1").await;
        dependency("database", result.map(|_| ()))
    }

    async fn migrations(&self) -> ReadinessDependency {
        let result = DynamicMigrator::get_pending_migrations(&self.database)
            .await
            .and_then(|pending| {
                if pending.is_empty() {
                    Ok(())
                } else {
                    Err(sea_orm::DbErr::Migration(format!(
                        "{} pending migrations",
                        pending.len()
                    )))
                }
            });
        dependency("migrations", result)
    }
}

#[async_trait::async_trait]
impl ReadinessProvider for RuntimeReadiness {
    async fn check(&self) -> ReadinessReport {
        let (database, migrations, embedding, vector_store) = tokio::join!(
            self.database(),
            self.migrations(),
            async { dependency("embedding", self.embedding.health_check().await) },
            async { dependency("vector_store", self.vector_store.health_check().await) },
        );
        let dependencies = vec![database, embedding, vector_store, migrations];
        let ready = dependencies.iter().all(|item| item.ready);
        ReadinessReport {
            ready,
            dependencies,
        }
    }
}

fn dependency<E: std::fmt::Display>(
    name: &'static str,
    result: std::result::Result<(), E>,
) -> ReadinessDependency {
    match result {
        Ok(()) => ReadinessDependency {
            name,
            ready: true,
            error: None,
        },
        Err(error) => ReadinessDependency {
            name,
            ready: false,
            error: Some(error.to_string()),
        },
    }
}
