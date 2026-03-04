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

    /// Persists a new project phase.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_phase(&self, phase: &ProjectPhase) -> Result<()> {
        sea_repo_insert!(&self.db, project_phase, phase, "create project phase")
    }

    /// Fetches a project phase by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the phase is not found.
    pub async fn get_phase_by_id(&self, id: &str) -> Result<ProjectPhase> {
        sea_repo_get!(
            &self.db,
            project_phase,
            ProjectPhase,
            "ProjectPhase",
            id,
            "get project phase by id"
        )
    }

    /// Lists all phases for a project ordered by sequence.
    ///
    /// # Errors
    ///
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

    /// Updates an existing project phase.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn update_phase(&self, phase: &ProjectPhase) -> Result<()> {
        sea_repo_update!(&self.db, project_phase, phase, "update project phase")
    }

    /// Deletes a project phase by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete fails.
    pub async fn delete_phase(&self, id: &str) -> Result<()> {
        sea_repo_delete!(&self.db, project_phase, id, "delete project phase")
    }

    /// Persists a new project issue.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_repo_insert!(&self.db, project_issue, issue, "create project issue")
    }

    /// Fetches a project issue by organization and issue id.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the issue is not found.
    pub async fn get_issue_by_id(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        sea_repo_get_filtered!(&self.db, project_issue, ProjectIssue, "ProjectIssue", id, "get project issue by id",
            project_issue::Column::OrgId => org_id)
    }

    /// Lists all issues for a project in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        let models = project_issue::Entity::find()
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .filter(project_issue::Column::ProjectId.eq(project_id.to_owned()))
            .order_by_asc(project_issue::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_error("list project issues"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    /// Lists issues using a rich filter.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or the project is not found.
    pub async fn list_issues_filtered(
        &self,
        org_id: &str,
        filter: &IssueFilter,
    ) -> Result<Vec<ProjectIssue>> {
        let mut q = project_issue::Entity::find()
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .order_by_desc(project_issue::Column::UpdatedAt);

        if let Some(project_id) = &filter.project_id {
            let project_exists = project::Entity::find()
                .filter(project::Column::Id.eq(project_id.as_str()))
                .filter(project::Column::OrgId.eq(org_id))
                .count(&self.db)
                .await
                .map_err(db_error("verify project ownership"))?
                > 0;
            if !project_exists {
                return Err(Error::not_found(format!(
                    "Project with id {project_id} not found in org"
                )));
            }
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

        let models = q
            .all(&self.db)
            .await
            .map_err(db_error("list filtered project issues"))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    /// Updates an existing project issue.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_repo_update!(&self.db, project_issue, issue, "update project issue")
    }

    /// Deletes an issue by organization and id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete fails.
    pub async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        sea_repo_delete_filtered!(&self.db, project_issue, id, "delete project issue", project_issue::Column::OrgId => org_id.to_owned())
    }

    /// Persists a dependency edge between issues.
    ///
    /// # Errors
    ///
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
    ///
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
    ///
    /// Returns an error if any dependency query fails.
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
    ///
    /// Returns an error if the database delete fails.
    pub async fn delete_dependency(&self, id: &str) -> Result<()> {
        sea_repo_delete!(
            &self.db,
            project_dependency,
            id,
            "delete project dependency"
        )
    }

    /// Persists a new project decision.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_decision(&self, decision: &ProjectDecision) -> Result<()> {
        sea_repo_insert!(
            &self.db,
            project_decision,
            decision,
            "create project decision"
        )
    }

    /// Fetches a project decision by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or the decision is not found.
    pub async fn get_decision_by_id(&self, id: &str) -> Result<ProjectDecision> {
        sea_repo_get!(
            &self.db,
            project_decision,
            ProjectDecision,
            "ProjectDecision",
            id,
            "get project decision by id"
        )
    }

    /// Lists all decisions for a project.
    ///
    /// # Errors
    ///
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

    /// Updates an existing project decision.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn update_decision(&self, decision: &ProjectDecision) -> Result<()> {
        sea_repo_update!(
            &self.db,
            project_decision,
            decision,
            "update project decision"
        )
    }

    /// Deletes a project decision by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete fails.
    pub async fn delete_decision(&self, id: &str) -> Result<()> {
        sea_repo_delete!(&self.db, project_decision, id, "delete project decision")
    }
}

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    async fn create(&self, project: &Project) -> Result<()> {
        sea_repo_insert!(&self.db, project, project, "create project")
    }

    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project> {
        sea_repo_get_filtered!(&self.db, project, Project, "Project", id, "get project by id",
            project::Column::OrgId => org_id)
    }

    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project> {
        let model = project::Entity::find()
            .filter(project::Column::OrgId.eq(org_id.to_owned()))
            .filter(project::Column::Name.eq(name.to_owned()))
            .one(&self.db)
            .await
            .map_err(db_error("get project by name"))?;

        Error::not_found_or(model.map(Into::into), "Project", name)
    }

    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project> {
        let model = project::Entity::find()
            .filter(project::Column::OrgId.eq(org_id.to_owned()))
            .filter(project::Column::Path.eq(path.to_owned()))
            .one(&self.db)
            .await
            .map_err(db_error("get project by path"))?;

        Error::not_found_or(model.map(Into::into), "Project", path)
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
        sea_repo_delete_filtered!(&self.db, project, id, "delete project", project::Column::OrgId => org_id.to_owned())
    }
}
