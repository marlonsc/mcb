//! SeaORM-backed project repository implementation.

use std::collections::{HashSet, VecDeque};

use async_trait::async_trait;
use mcb_domain::entities::Project;
use mcb_domain::entities::project::{
    IssueFilter, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ProjectRepository;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use super::common::db_error;
use crate::database::seaorm::entities::{
    project, project_decision, project_dependency, project_issue, project_phase,
};

/// SeaORM-backed repository for project planning entities.
pub struct SeaOrmProjectRepository {
    db: DatabaseConnection,
}

impl SeaOrmProjectRepository {
    #[must_use]
    /// Creates a new project repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

// ── Phase CRUD (simple id-based) ────────────────────────────────────────────

sea_pub_crud!(SeaOrmProjectRepository {
    db_field: db,
    entity: project_phase,
    domain: ProjectPhase,
    label: "ProjectPhase",
    create: create_phase(phase),
    get: get_phase_by_id(id),
    update: update_phase(phase),
    delete: delete_phase(id),
});

impl SeaOrmProjectRepository {
    /// Lists all phases for a project ordered by sequence.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
        let models = project_phase::Entity::find()
            .filter(project_phase::Column::ProjectId.eq(project_id.to_owned()))
            .order_by_asc(project_phase::Column::Sequence)
            .order_by_asc(project_phase::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_error("list project phases"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }
}

// ── Decision CRUD (simple id-based) ─────────────────────────────────────────

sea_pub_crud!(SeaOrmProjectRepository {
    db_field: db,
    entity: project_decision,
    domain: ProjectDecision,
    label: "ProjectDecision",
    create: create_decision(decision),
    get: get_decision_by_id(id),
    update: update_decision(decision),
    delete: delete_decision(id),
});

impl SeaOrmProjectRepository {
    /// Lists all decisions for a project ordered by creation date.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
        let models = project_decision::Entity::find()
            .filter(project_decision::Column::ProjectId.eq(project_id.to_owned()))
            .order_by_desc(project_decision::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_error("list project decisions"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }
}

// ── Issue CRUD (org-scoped) ─────────────────────────────────────────────────

sea_pub_crud_scoped!(SeaOrmProjectRepository {
    db_field: db, entity: project_issue, domain: ProjectIssue,
    label: "ProjectIssue",
    scope_col: project_issue::Column::OrgId,
    create: create_issue(issue),
    get: get_issue_by_id,
    list: list_issues(project_issue::Column::ProjectId => project_id),
    update: update_issue(issue),
    delete: delete_issue,
});

impl SeaOrmProjectRepository {
    /// Verifies a project exists in the given org.
    async fn verify_project_exists(&self, org_id: &str, project_id: &str) -> Result<()> {
        let exists = project::Entity::find()
            .filter(project::Column::Id.eq(project_id))
            .filter(project::Column::OrgId.eq(org_id))
            .count(&self.db)
            .await
            .map_err(db_error("verify project ownership"))?
            > 0;
        if !exists {
            return Err(Error::not_found(format!(
                "Project with id {project_id} not found in org"
            )));
        }
        Ok(())
    }

    /// Applies `IssueFilter` fields to a `SeaORM` `Select` query.
    fn apply_issue_filter(
        mut q: sea_orm::Select<project_issue::Entity>,
        filter: &IssueFilter,
    ) -> sea_orm::Select<project_issue::Entity> {
        if let Some(project_id) = &filter.project_id {
            q = q.filter(project_issue::Column::ProjectId.eq(project_id.to_owned()));
        }
        if let Some(phase_id) = &filter.phase_id {
            q = q.filter(project_issue::Column::PhaseId.eq(phase_id.to_owned()));
        }
        if let Some(issue_type) = &filter.issue_type {
            q = q.filter(project_issue::Column::IssueType.eq(issue_type.to_string()));
        }
        if let Some(status) = &filter.status {
            q = q.filter(project_issue::Column::Status.eq(status.to_string()));
        }
        if let Some(priority) = filter.priority {
            q = q.filter(project_issue::Column::Priority.eq(i64::from(priority)));
        }
        if let Some(assignee) = &filter.assignee {
            q = q.filter(project_issue::Column::Assignee.eq(assignee.to_owned()));
        }
        if let Some(label) = &filter.label {
            let like_pattern = format!("%\"{label}\"%");
            q = q.filter(project_issue::Column::Labels.like(like_pattern));
        }
        if let Some(limit) = filter.limit {
            q = q.limit(limit as u64);
        }
        q
    }

    /// Lists issues using a rich filter.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn list_issues_filtered(
        &self,
        org_id: &str,
        filter: &IssueFilter,
    ) -> Result<Vec<ProjectIssue>> {
        if let Some(project_id) = &filter.project_id {
            self.verify_project_exists(org_id, project_id).await?;
        }

        let q = project_issue::Entity::find()
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .order_by_desc(project_issue::Column::UpdatedAt);

        let models = Self::apply_issue_filter(q, filter)
            .all(&self.db)
            .await
            .map_err(db_error("list filtered project issues"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }
}

// ── Dependency CRUD ─────────────────────────────────────────────────────────

impl SeaOrmProjectRepository {
    /// Persists a dependency edge between issues.
    ///
    /// # Errors
    /// Returns an error if the database insert fails.
    pub async fn create_dependency(&self, dependency: &ProjectDependency) -> Result<()> {
        sea_repo_insert!(
            &self.db,
            project_dependency,
            dependency,
            "create project dependency"
        )
    }

    /// Lists all dependencies attached to an issue.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn list_dependencies(&self, issue_id: &str) -> Result<Vec<ProjectDependency>> {
        let models = project_dependency::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(project_dependency::Column::FromIssueId.eq(issue_id.to_owned()))
                    .add(project_dependency::Column::ToIssueId.eq(issue_id.to_owned())),
            )
            .order_by_asc(project_dependency::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_error("list project dependencies"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    /// Traverses issue dependencies breadth-first up to `max_depth`.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn traverse_dependencies(
        &self,
        issue_id: &str,
        max_depth: usize,
    ) -> Result<Vec<ProjectDependency>> {
        let mut queue = VecDeque::new();
        queue.push_back((issue_id.to_owned(), 0usize));

        let mut visited_nodes = HashSet::new();
        visited_nodes.insert(issue_id.to_owned());

        let mut visited_edges = HashSet::new();
        let mut traversed = Vec::new();

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let models = project_dependency::Entity::find()
                .filter(project_dependency::Column::FromIssueId.eq(current))
                .order_by_asc(project_dependency::Column::CreatedAt)
                .all(&self.db)
                .await
                .map_err(db_error("traverse project dependencies"))?;

            for model in models {
                let dependency: ProjectDependency = model.into();
                if visited_edges.insert(dependency.id.clone()) {
                    let next = dependency.to_issue_id.clone();
                    traversed.push(dependency);
                    if visited_nodes.insert(next.clone()) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }

        Ok(traversed)
    }

    /// Deletes a dependency edge by id.
    ///
    /// # Errors
    /// Returns an error if the database delete fails.
    pub async fn delete_dependency(&self, id: &str) -> Result<()> {
        sea_repo_delete!(
            &self.db,
            project_dependency,
            id,
            "delete project dependency"
        )
    }
}

// ── ProjectRepository trait impl ────────────────────────────────────────────

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    async fn create(&self, project: &Project) -> Result<()> {
        sea_repo_insert!(&self.db, project, project, "create project")
    }

    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project> {
        sea_repo_get_filtered!(&self.db, project, Project, "Project", id,
            "get project by id", project::Column::OrgId => org_id)
    }

    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project> {
        sea_repo_find_by_column!(&self.db, project, Project, "Project", name,
            "get project by name",
            project::Column::OrgId => org_id,
            project::Column::Name => name)
    }

    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project> {
        sea_repo_find_by_column!(&self.db, project, Project, "Project", path,
            "get project by path",
            project::Column::OrgId => org_id,
            project::Column::Path => path)
    }

    async fn list(&self, org_id: &str) -> Result<Vec<Project>> {
        let models = project::Entity::find()
            .filter(project::Column::OrgId.eq(org_id.to_owned()))
            .order_by_asc(project::Column::Name)
            .all(&self.db)
            .await
            .map_err(db_error("list projects"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn update(&self, project: &Project) -> Result<()> {
        sea_repo_update!(&self.db, project, project, "update project")
    }

    async fn delete(&self, org_id: &str, id: &str) -> Result<()> {
        sea_repo_delete_filtered!(&self.db, project, id, "delete project",
            project::Column::OrgId => org_id.to_owned())
    }
}
