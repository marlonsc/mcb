#![allow(clippy::missing_errors_doc)]

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
    QueryOrder, QuerySelect, Set,
};

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
    pub async fn create_phase(&self, phase: &ProjectPhase) -> Result<()> {
        let active: project_phase::ActiveModel = phase.clone().into();
        project_phase::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project phase"))?;
        Ok(())
    }

    /// Fetches a project phase by id.
    pub async fn get_phase_by_id(&self, id: &str) -> Result<ProjectPhase> {
        let model = project_phase::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(db_error("get project phase by id"))?;

        Error::not_found_or(model.map(Into::into), "ProjectPhase", id)
    }

    /// Lists all phases for a project ordered by sequence.
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
    pub async fn update_phase(&self, phase: &ProjectPhase) -> Result<()> {
        let active = project_phase::ActiveModel {
            id: Set(phase.id.clone()),
            project_id: Set(phase.project_id.clone()),
            name: Set(phase.name.clone()),
            description: Set(phase.description.clone()),
            sequence: Set(i64::from(phase.sequence)),
            status: Set(phase.status.to_string()),
            started_at: Set(phase.started_at),
            completed_at: Set(phase.completed_at),
            created_at: Set(phase.created_at),
            updated_at: Set(phase.updated_at),
        };
        active
            .update(&self.db)
            .await
            .map_err(db_error("update project phase"))?;
        Ok(())
    }

    /// Deletes a project phase by id.
    pub async fn delete_phase(&self, id: &str) -> Result<()> {
        project_phase::Entity::delete_by_id(id.to_owned())
            .exec(&self.db)
            .await
            .map_err(db_error("delete project phase"))?;
        Ok(())
    }

    /// Persists a new project issue.
    pub async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        project_issue::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project issue"))?;
        Ok(())
    }

    /// Fetches a project issue by organization and issue id.
    pub async fn get_issue_by_id(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        let model = project_issue::Entity::find_by_id(id.to_owned())
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .one(&self.db)
            .await
            .map_err(db_error("get project issue by id"))?;

        Error::not_found_or(model.map(Into::into), "ProjectIssue", id)
    }

    /// Lists all issues for a project in an organization.
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
    pub async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        project_issue::Entity::update(active)
            .exec(&self.db)
            .await
            .map_err(db_error("update project issue"))?;
        Ok(())
    }

    /// Deletes an issue by organization and id.
    pub async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        project_issue::Entity::delete_many()
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .filter(project_issue::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map_err(db_error("delete project issue"))?;
        Ok(())
    }

    /// Persists a dependency edge between issues.
    pub async fn create_dependency(&self, dependency: &ProjectDependency) -> Result<()> {
        let active: project_dependency::ActiveModel = dependency.clone().into();
        project_dependency::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project dependency"))?;
        Ok(())
    }

    /// Lists all dependencies attached to an issue.
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
                .filter(project_dependency::Column::FromIssueId.eq(current.clone()))
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
    pub async fn delete_dependency(&self, id: &str) -> Result<()> {
        project_dependency::Entity::delete_by_id(id.to_owned())
            .exec(&self.db)
            .await
            .map_err(db_error("delete project dependency"))?;
        Ok(())
    }

    /// Persists a new project decision.
    pub async fn create_decision(&self, decision: &ProjectDecision) -> Result<()> {
        let active: project_decision::ActiveModel = decision.clone().into();
        project_decision::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project decision"))?;
        Ok(())
    }

    /// Fetches a project decision by id.
    pub async fn get_decision_by_id(&self, id: &str) -> Result<ProjectDecision> {
        let model = project_decision::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(db_error("get project decision by id"))?;

        Error::not_found_or(model.map(Into::into), "ProjectDecision", id)
    }

    /// Lists all decisions for a project.
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
    pub async fn update_decision(&self, decision: &ProjectDecision) -> Result<()> {
        let active = project_decision::ActiveModel {
            id: Set(decision.id.clone()),
            project_id: Set(decision.project_id.clone()),
            issue_id: Set(decision.issue_id.clone()),
            title: Set(decision.title.clone()),
            context: Set(decision.context.clone()),
            decision: Set(decision.decision.clone()),
            consequences: Set(decision.consequences.clone()),
            created_at: Set(decision.created_at),
        };
        active
            .update(&self.db)
            .await
            .map_err(db_error("update project decision"))?;
        Ok(())
    }

    /// Deletes a project decision by id.
    pub async fn delete_decision(&self, id: &str) -> Result<()> {
        project_decision::Entity::delete_by_id(id.to_owned())
            .exec(&self.db)
            .await
            .map_err(db_error("delete project decision"))?;
        Ok(())
    }
}

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    async fn create(&self, project: &Project) -> Result<()> {
        let active: project::ActiveModel = project.clone().into();
        project::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project"))?;
        Ok(())
    }

    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project> {
        let model = project::Entity::find_by_id(id.to_owned())
            .filter(project::Column::OrgId.eq(org_id.to_owned()))
            .one(&self.db)
            .await
            .map_err(db_error("get project by id"))?;

        Error::not_found_or(model.map(Into::into), "Project", id)
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
        let active: project::ActiveModel = project.clone().into();
        active
            .update(&self.db)
            .await
            .map_err(db_error("update project"))?;
        Ok(())
    }

    async fn delete(&self, org_id: &str, id: &str) -> Result<()> {
        project::Entity::delete_many()
            .filter(project::Column::OrgId.eq(org_id.to_owned()))
            .filter(project::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map_err(db_error("delete project"))?;
        Ok(())
    }
}

fn db_error(op: &'static str) -> impl Fn(sea_orm::DbErr) -> Error {
    move |e| Error::database(format!("{op}: {e}"))
}
