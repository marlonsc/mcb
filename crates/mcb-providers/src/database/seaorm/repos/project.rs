use std::collections::{HashSet, VecDeque};

use async_trait::async_trait;
use mcb_domain::entities::Project;
use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ProjectRepository;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Order, Query, SqliteQueryBuilder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Statement,
};

use crate::database::seaorm::entities::{project, project_issue};

pub struct SeaOrmProjectRepository {
    db: DatabaseConnection,
}

impl SeaOrmProjectRepository {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_phase(&self, phase: &ProjectPhase) -> Result<()> {
        self.db
            .execute(&stmt(
                "INSERT INTO project_phases (id, project_id, name, description, sequence, status, started_at, completed_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                vec![
                    phase.id.clone().into(),
                    phase.project_id.clone().into(),
                    phase.name.clone().into(),
                    phase.description.clone().into(),
                    i64::from(phase.sequence).into(),
                    phase.status.to_string().into(),
                    phase.started_at.into(),
                    phase.completed_at.into(),
                    phase.created_at.into(),
                    phase.updated_at.into(),
                ],
            ))
            .await
            .map_err(db_error("create project phase"))?;
        Ok(())
    }

    pub async fn get_phase_by_id(&self, id: &str) -> Result<ProjectPhase> {
        let row = self
            .db
            .query_one(&stmt(
                "SELECT id, project_id, name, description, sequence, status, started_at, completed_at, created_at, updated_at FROM project_phases WHERE id = ? LIMIT 1",
                vec![id.into()],
            ))
            .await
            .map_err(db_error("get project phase by id"))?;

        let Some(row) = row else {
            return Err(Error::not_found(format!("ProjectPhase {id}")));
        };

        row_to_phase(&row)
    }

    pub async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
        let mut select = Query::select();
        select
            .columns([
                Alias::new("id"),
                Alias::new("project_id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("sequence"),
                Alias::new("status"),
                Alias::new("started_at"),
                Alias::new("completed_at"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .from(Alias::new("project_phases"))
            .and_where(Expr::col(Alias::new("project_id")).eq(project_id))
            .order_by(Alias::new("sequence"), Order::Asc)
            .order_by(Alias::new("created_at"), Order::Asc);

        let (sql, values) = select.build(SqliteQueryBuilder);
        let rows = self
            .db
            .query_all(&Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(db_error("list project phases"))?;

        rows.into_iter().map(|row| row_to_phase(&row)).collect()
    }

    pub async fn update_phase(&self, phase: &ProjectPhase) -> Result<()> {
        self.db
            .execute(&stmt(
                "UPDATE project_phases SET name = ?, description = ?, sequence = ?, status = ?, started_at = ?, completed_at = ?, updated_at = ? WHERE id = ?",
                vec![
                    phase.name.clone().into(),
                    phase.description.clone().into(),
                    i64::from(phase.sequence).into(),
                    phase.status.to_string().into(),
                    phase.started_at.into(),
                    phase.completed_at.into(),
                    phase.updated_at.into(),
                    phase.id.clone().into(),
                ],
            ))
            .await
            .map_err(db_error("update project phase"))?;
        Ok(())
    }

    pub async fn delete_phase(&self, id: &str) -> Result<()> {
        self.db
            .execute(&stmt(
                "DELETE FROM project_phases WHERE id = ?",
                vec![id.into()],
            ))
            .await
            .map_err(db_error("delete project phase"))?;
        Ok(())
    }

    pub async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        project_issue::Entity::insert(active)
            .exec(&self.db)
            .await
            .map_err(db_error("create project issue"))?;
        Ok(())
    }

    pub async fn get_issue_by_id(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        let model = project_issue::Entity::find_by_id(id.to_owned())
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .one(&self.db)
            .await
            .map_err(db_error("get project issue by id"))?;

        Error::not_found_or(model.map(Into::into), "ProjectIssue", id)
    }

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

    pub async fn list_issues_filtered(
        &self,
        org_id: &str,
        filter: &IssueFilter,
    ) -> Result<Vec<ProjectIssue>> {
        let mut select = Query::select();
        select
            .columns([
                Alias::new("id"),
                Alias::new("org_id"),
                Alias::new("project_id"),
                Alias::new("phase_id"),
                Alias::new("title"),
                Alias::new("description"),
                Alias::new("issue_type"),
                Alias::new("status"),
                Alias::new("priority"),
                Alias::new("assignee"),
                Alias::new("labels"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
                Alias::new("closed_at"),
                Alias::new("created_by"),
                Alias::new("estimated_minutes"),
                Alias::new("actual_minutes"),
                Alias::new("notes"),
                Alias::new("design"),
                Alias::new("parent_issue_id"),
                Alias::new("closed_reason"),
            ])
            .from(Alias::new("project_issues"))
            .and_where(Expr::col(Alias::new("org_id")).eq(org_id))
            .order_by(Alias::new("updated_at"), Order::Desc);

        if let Some(project_id) = &filter.project_id {
            select.and_where(Expr::col(Alias::new("project_id")).eq(project_id));
        }
        if let Some(phase_id) = &filter.phase_id {
            select.and_where(Expr::col(Alias::new("phase_id")).eq(phase_id));
        }
        if let Some(issue_type) = &filter.issue_type {
            select.and_where(Expr::col(Alias::new("issue_type")).eq(issue_type.to_string()));
        }
        if let Some(status) = &filter.status {
            select.and_where(Expr::col(Alias::new("status")).eq(status.to_string()));
        }
        if let Some(priority) = filter.priority {
            select.and_where(Expr::col(Alias::new("priority")).eq(i64::from(priority)));
        }
        if let Some(assignee) = &filter.assignee {
            select.and_where(Expr::col(Alias::new("assignee")).eq(assignee));
        }
        if let Some(label) = &filter.label {
            let like_pattern = format!("%\"{label}\"%");
            select.and_where(Expr::col(Alias::new("labels")).like(like_pattern));
        }
        if let Some(limit) = filter.limit {
            select.limit(limit as u64);
        }

        let (sql, values) = select.build(SqliteQueryBuilder);
        let rows = self
            .db
            .query_all(&Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(db_error("list filtered project issues"))?;

        rows.into_iter().map(|row| row_to_issue(&row)).collect()
    }

    pub async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        project_issue::Entity::update(active)
            .exec(&self.db)
            .await
            .map_err(db_error("update project issue"))?;
        Ok(())
    }

    pub async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        project_issue::Entity::delete_many()
            .filter(project_issue::Column::OrgId.eq(org_id.to_owned()))
            .filter(project_issue::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map_err(db_error("delete project issue"))?;
        Ok(())
    }

    pub async fn create_dependency(&self, dependency: &ProjectDependency) -> Result<()> {
        self.db
            .execute(&stmt(
                "INSERT INTO project_dependencies (id, from_issue_id, to_issue_id, dependency_type, created_at) VALUES (?, ?, ?, ?, ?)",
                vec![
                    dependency.id.clone().into(),
                    dependency.from_issue_id.clone().into(),
                    dependency.to_issue_id.clone().into(),
                    dependency.dependency_type.to_string().into(),
                    dependency.created_at.into(),
                ],
            ))
            .await
            .map_err(db_error("create project dependency"))?;
        Ok(())
    }

    pub async fn list_dependencies(&self, issue_id: &str) -> Result<Vec<ProjectDependency>> {
        let mut select = Query::select();
        select
            .columns([
                Alias::new("id"),
                Alias::new("from_issue_id"),
                Alias::new("to_issue_id"),
                Alias::new("dependency_type"),
                Alias::new("created_at"),
            ])
            .from(Alias::new("project_dependencies"))
            .and_where(
                Expr::col(Alias::new("from_issue_id"))
                    .eq(issue_id)
                    .or(Expr::col(Alias::new("to_issue_id")).eq(issue_id)),
            )
            .order_by(Alias::new("created_at"), Order::Asc);

        let (sql, values) = select.build(SqliteQueryBuilder);
        let rows = self
            .db
            .query_all(&Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(db_error("list project dependencies"))?;

        rows.into_iter()
            .map(|row| row_to_dependency(&row))
            .collect()
    }

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

            let mut select = Query::select();
            select
                .columns([
                    Alias::new("id"),
                    Alias::new("from_issue_id"),
                    Alias::new("to_issue_id"),
                    Alias::new("dependency_type"),
                    Alias::new("created_at"),
                ])
                .from(Alias::new("project_dependencies"))
                .and_where(Expr::col(Alias::new("from_issue_id")).eq(current.clone()))
                .order_by(Alias::new("created_at"), Order::Asc);

            let (sql, values) = select.build(SqliteQueryBuilder);
            let rows = self
                .db
                .query_all(&Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    sql,
                    values,
                ))
                .await
                .map_err(db_error("traverse project dependencies"))?;

            for row in rows {
                let dependency = row_to_dependency(&row)?;
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

    pub async fn delete_dependency(&self, id: &str) -> Result<()> {
        self.db
            .execute(&stmt(
                "DELETE FROM project_dependencies WHERE id = ?",
                vec![id.into()],
            ))
            .await
            .map_err(db_error("delete project dependency"))?;
        Ok(())
    }

    pub async fn create_decision(&self, decision: &ProjectDecision) -> Result<()> {
        self.db
            .execute(&stmt(
                "INSERT INTO project_decisions (id, project_id, issue_id, title, context, decision, consequences, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                vec![
                    decision.id.clone().into(),
                    decision.project_id.clone().into(),
                    decision.issue_id.clone().into(),
                    decision.title.clone().into(),
                    decision.context.clone().into(),
                    decision.decision.clone().into(),
                    decision.consequences.clone().into(),
                    decision.created_at.into(),
                ],
            ))
            .await
            .map_err(db_error("create project decision"))?;
        Ok(())
    }

    pub async fn get_decision_by_id(&self, id: &str) -> Result<ProjectDecision> {
        let row = self
            .db
            .query_one(&stmt(
                "SELECT id, project_id, issue_id, title, context, decision, consequences, created_at FROM project_decisions WHERE id = ? LIMIT 1",
                vec![id.into()],
            ))
            .await
            .map_err(db_error("get project decision by id"))?;

        let Some(row) = row else {
            return Err(Error::not_found(format!("ProjectDecision {id}")));
        };

        row_to_decision(&row)
    }

    pub async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
        let mut select = Query::select();
        select
            .columns([
                Alias::new("id"),
                Alias::new("project_id"),
                Alias::new("issue_id"),
                Alias::new("title"),
                Alias::new("context"),
                Alias::new("decision"),
                Alias::new("consequences"),
                Alias::new("created_at"),
            ])
            .from(Alias::new("project_decisions"))
            .and_where(Expr::col(Alias::new("project_id")).eq(project_id))
            .order_by(Alias::new("created_at"), Order::Desc);

        let (sql, values) = select.build(SqliteQueryBuilder);
        let rows = self
            .db
            .query_all(&Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(db_error("list project decisions"))?;

        rows.into_iter().map(|row| row_to_decision(&row)).collect()
    }

    pub async fn update_decision(&self, decision: &ProjectDecision) -> Result<()> {
        self.db
            .execute(&stmt(
                "UPDATE project_decisions SET issue_id = ?, title = ?, context = ?, decision = ?, consequences = ? WHERE id = ?",
                vec![
                    decision.issue_id.clone().into(),
                    decision.title.clone().into(),
                    decision.context.clone().into(),
                    decision.decision.clone().into(),
                    decision.consequences.clone().into(),
                    decision.id.clone().into(),
                ],
            ))
            .await
            .map_err(db_error("update project decision"))?;
        Ok(())
    }

    pub async fn delete_decision(&self, id: &str) -> Result<()> {
        self.db
            .execute(&stmt(
                "DELETE FROM project_decisions WHERE id = ?",
                vec![id.into()],
            ))
            .await
            .map_err(db_error("delete project decision"))?;
        Ok(())
    }
}

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    async fn create(&self, project: &Project) -> Result<()> {
        project::Entity::insert(project.clone().into())
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
        project::Entity::update(project.clone().into())
            .exec(&self.db)
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

fn stmt(sql: &str, values: Vec<sea_orm::Value>) -> Statement {
    Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, values)
}

fn db_error(op: &'static str) -> impl Fn(sea_orm::DbErr) -> Error {
    move |e| Error::database(format!("{op}: {e}"))
}

fn row_to_phase(row: &sea_orm::QueryResult) -> Result<ProjectPhase> {
    let status = row
        .try_get_by::<String>("status")
        .map_err(|e| Error::database(format!("decode phase status: {e}")))?
        .parse::<PhaseStatus>()
        .map_err(|e| Error::invalid_argument(format!("invalid phase status: {e}")))?;

    Ok(ProjectPhase {
        id: row
            .try_get_by::<String>("id")
            .map_err(|e| Error::database(format!("decode phase id: {e}")))?,
        project_id: row
            .try_get_by::<String>("project_id")
            .map_err(|e| Error::database(format!("decode phase project_id: {e}")))?,
        name: row
            .try_get_by::<String>("name")
            .map_err(|e| Error::database(format!("decode phase name: {e}")))?,
        description: row
            .try_get_by::<String>("description")
            .map_err(|e| Error::database(format!("decode phase description: {e}")))?,
        sequence: row
            .try_get_by::<i64>("sequence")
            .map_err(|e| Error::database(format!("decode phase sequence: {e}")))?
            as i32,
        status,
        started_at: row
            .try_get_by::<Option<i64>>("started_at")
            .map_err(|e| Error::database(format!("decode phase started_at: {e}")))?,
        completed_at: row
            .try_get_by::<Option<i64>>("completed_at")
            .map_err(|e| Error::database(format!("decode phase completed_at: {e}")))?,
        created_at: row
            .try_get_by::<i64>("created_at")
            .map_err(|e| Error::database(format!("decode phase created_at: {e}")))?,
        updated_at: row
            .try_get_by::<i64>("updated_at")
            .map_err(|e| Error::database(format!("decode phase updated_at: {e}")))?,
    })
}

fn row_to_issue(row: &sea_orm::QueryResult) -> Result<ProjectIssue> {
    let labels_json = row
        .try_get_by::<String>("labels")
        .map_err(|e| Error::database(format!("decode issue labels: {e}")))?;
    let labels: Vec<String> = serde_json::from_str(&labels_json)
        .map_err(|e| Error::database(format!("decode issue labels json: {e}")))?;

    let issue_type = row
        .try_get_by::<String>("issue_type")
        .map_err(|e| Error::database(format!("decode issue_type: {e}")))?
        .parse::<IssueType>()
        .map_err(|e| Error::invalid_argument(format!("invalid issue_type: {e}")))?;

    let status = row
        .try_get_by::<String>("status")
        .map_err(|e| Error::database(format!("decode issue status: {e}")))?
        .parse::<IssueStatus>()
        .map_err(|e| Error::invalid_argument(format!("invalid issue status: {e}")))?;

    Ok(ProjectIssue {
        id: row
            .try_get_by::<String>("id")
            .map_err(|e| Error::database(format!("decode issue id: {e}")))?,
        org_id: row
            .try_get_by::<String>("org_id")
            .map_err(|e| Error::database(format!("decode issue org_id: {e}")))?,
        project_id: row
            .try_get_by::<String>("project_id")
            .map_err(|e| Error::database(format!("decode issue project_id: {e}")))?,
        phase_id: row
            .try_get_by::<Option<String>>("phase_id")
            .map_err(|e| Error::database(format!("decode issue phase_id: {e}")))?,
        title: row
            .try_get_by::<String>("title")
            .map_err(|e| Error::database(format!("decode issue title: {e}")))?,
        description: row
            .try_get_by::<String>("description")
            .map_err(|e| Error::database(format!("decode issue description: {e}")))?,
        issue_type,
        status,
        priority: row
            .try_get_by::<i64>("priority")
            .map_err(|e| Error::database(format!("decode issue priority: {e}")))?
            as i32,
        assignee: row
            .try_get_by::<Option<String>>("assignee")
            .map_err(|e| Error::database(format!("decode issue assignee: {e}")))?,
        labels,
        created_at: row
            .try_get_by::<i64>("created_at")
            .map_err(|e| Error::database(format!("decode issue created_at: {e}")))?,
        updated_at: row
            .try_get_by::<i64>("updated_at")
            .map_err(|e| Error::database(format!("decode issue updated_at: {e}")))?,
        closed_at: row
            .try_get_by::<Option<i64>>("closed_at")
            .map_err(|e| Error::database(format!("decode issue closed_at: {e}")))?,
        created_by: row
            .try_get_by::<String>("created_by")
            .map_err(|e| Error::database(format!("decode issue created_by: {e}")))?,
        estimated_minutes: row
            .try_get_by::<Option<i64>>("estimated_minutes")
            .map_err(|e| Error::database(format!("decode issue estimated_minutes: {e}")))?,
        actual_minutes: row
            .try_get_by::<Option<i64>>("actual_minutes")
            .map_err(|e| Error::database(format!("decode issue actual_minutes: {e}")))?,
        notes: row
            .try_get_by::<String>("notes")
            .map_err(|e| Error::database(format!("decode issue notes: {e}")))?,
        design: row
            .try_get_by::<String>("design")
            .map_err(|e| Error::database(format!("decode issue design: {e}")))?,
        parent_issue_id: row
            .try_get_by::<Option<String>>("parent_issue_id")
            .map_err(|e| Error::database(format!("decode issue parent_issue_id: {e}")))?,
        closed_reason: row
            .try_get_by::<String>("closed_reason")
            .map_err(|e| Error::database(format!("decode issue closed_reason: {e}")))?,
    })
}

fn row_to_dependency(row: &sea_orm::QueryResult) -> Result<ProjectDependency> {
    let dependency_type = row
        .try_get_by::<String>("dependency_type")
        .map_err(|e| Error::database(format!("decode dependency_type: {e}")))?
        .parse::<DependencyType>()
        .map_err(|e| Error::invalid_argument(format!("invalid dependency_type: {e}")))?;

    Ok(ProjectDependency {
        id: row
            .try_get_by::<String>("id")
            .map_err(|e| Error::database(format!("decode dependency id: {e}")))?,
        from_issue_id: row
            .try_get_by::<String>("from_issue_id")
            .map_err(|e| Error::database(format!("decode from_issue_id: {e}")))?,
        to_issue_id: row
            .try_get_by::<String>("to_issue_id")
            .map_err(|e| Error::database(format!("decode to_issue_id: {e}")))?,
        dependency_type,
        created_at: row
            .try_get_by::<i64>("created_at")
            .map_err(|e| Error::database(format!("decode dependency created_at: {e}")))?,
    })
}

fn row_to_decision(row: &sea_orm::QueryResult) -> Result<ProjectDecision> {
    Ok(ProjectDecision {
        id: row
            .try_get_by::<String>("id")
            .map_err(|e| Error::database(format!("decode decision id: {e}")))?,
        project_id: row
            .try_get_by::<String>("project_id")
            .map_err(|e| Error::database(format!("decode decision project_id: {e}")))?,
        issue_id: row
            .try_get_by::<Option<String>>("issue_id")
            .map_err(|e| Error::database(format!("decode decision issue_id: {e}")))?,
        title: row
            .try_get_by::<String>("title")
            .map_err(|e| Error::database(format!("decode decision title: {e}")))?,
        context: row
            .try_get_by::<String>("context")
            .map_err(|e| Error::database(format!("decode decision context: {e}")))?,
        decision: row
            .try_get_by::<String>("decision")
            .map_err(|e| Error::database(format!("decode decision decision: {e}")))?,
        consequences: row
            .try_get_by::<String>("consequences")
            .map_err(|e| Error::database(format!("decode decision consequences: {e}")))?,
        created_at: row
            .try_get_by::<i64>("created_at")
            .map_err(|e| Error::database(format!("decode decision created_at: {e}")))?,
    })
}
