//! SeaORM database repository factory registration.

use std::any::Any;
use std::sync::Arc;

use mcb_domain::registry::database::{
    DATABASE_REPOSITORY_PROVIDERS, DatabaseRepositories, DatabaseRepositoryEntry,
};
use sea_orm::DatabaseConnection;

use crate::database::seaorm::auth_repository::SeaOrmAuthRepositoryAdapter;
use crate::database::seaorm::dashboard::SeaOrmDashboardAdapter;
use crate::database::seaorm::repos::{
    SeaOrmAgentRepository, SeaOrmEntityRepository, SeaOrmIndexRepository,
    SeaOrmObservationRepository, SeaOrmProjectRepository,
};

/// Creates the complete SeaORM-backed repository bundle for the database registry.
///
/// # Errors
///
/// Returns an error when the boxed connection is not a `sea_orm::DatabaseConnection`.
fn create_seaorm_repositories(
    connection: Arc<dyn Any + Send + Sync>,
    project_id: String,
) -> mcb_domain::error::Result<DatabaseRepositories> {
    let db = connection.downcast::<DatabaseConnection>().map_err(|_| {
        mcb_domain::error::Error::configuration(
            "Expected sea_orm::DatabaseConnection but received different type",
        )
    })?;
    let db = db;

    let observation_repo = SeaOrmObservationRepository::new((*db).clone());
    let agent_repo = SeaOrmAgentRepository::new(Arc::clone(&db));
    let project_repo = SeaOrmProjectRepository::new((*db).clone());
    let entity_repo = Arc::new(SeaOrmEntityRepository::new(Arc::clone(&db)));
    let index_repo = SeaOrmIndexRepository::new(Arc::clone(&db), project_id);

    Ok(DatabaseRepositories {
        memory: Arc::new(observation_repo),
        auth: Arc::new(SeaOrmAuthRepositoryAdapter::new((*db).clone())),
        dashboard: Arc::new(SeaOrmDashboardAdapter::new((*db).clone())),
        agent: Arc::new(agent_repo),
        project: Arc::new(project_repo),
        vcs_entity: Arc::clone(&entity_repo) as _,
        plan_entity: Arc::clone(&entity_repo) as _,
        issue_entity: Arc::clone(&entity_repo) as _,
        org_entity: Arc::clone(&entity_repo) as _,
        file_hash: Arc::new(index_repo),
    })
}

/// `SeaORM` repository bundle provider registration.
#[linkme::distributed_slice(DATABASE_REPOSITORY_PROVIDERS)]
static SEAORM_REPOS: DatabaseRepositoryEntry = DatabaseRepositoryEntry {
    name: "seaorm",
    description: "SeaORM database repositories (SQLite, PostgreSQL, MySQL)",
    build: create_seaorm_repositories
        as fn(
            Arc<dyn Any + Send + Sync>,
            String,
        ) -> mcb_domain::error::Result<DatabaseRepositories>,
};
