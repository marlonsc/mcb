#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{AgentSessionQuery, AgentSessionRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use crate::database::seaorm::entities::agent_session;

pub struct SeaOrmAgentSessionRepository {
    db: Arc<DatabaseConnection>,
}

pub struct SessionSummaryView {
    pub session_id: String,
    pub status: AgentSessionStatus,
    pub agent_type: AgentType,
    pub model: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
}

impl SeaOrmAgentSessionRepository {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn summarize_session(&self, id: &str) -> Result<Option<SessionSummaryView>> {
        let model = agent_session::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(map_db_err)?;

        model
            .map(|record| {
                let domain = model_to_domain(record)?;
                Ok(SessionSummaryView {
                    session_id: domain.id,
                    status: domain.status,
                    agent_type: domain.agent_type,
                    model: domain.model,
                    started_at: domain.started_at,
                    ended_at: domain.ended_at,
                })
            })
            .transpose()
    }
}

#[allow(clippy::needless_pass_by_value)]
fn map_db_err(error: sea_orm::DbErr) -> Error {
    Error::database(format!("SeaORM agent session repository failure: {error}"))
}

fn validate_session_schema(session: &AgentSession) -> Result<()> {
    if session.model.trim().is_empty() {
        return Err(Error::invalid_argument(
            "Invalid session create schema: `model` must be non-empty",
        ));
    }

    if session.agent_type.as_str().trim().is_empty() {
        return Err(Error::invalid_argument(
            "Invalid session create schema: `agent_type` must be non-empty",
        ));
    }

    Ok(())
}

fn parse_agent_type(value: &str) -> Result<AgentType> {
    value
        .parse::<AgentType>()
        .map_err(|error| Error::invalid_argument(format!("Invalid session agent_type: {error}")))
}

fn parse_status(value: &str) -> Result<AgentSessionStatus> {
    value
        .parse::<AgentSessionStatus>()
        .map_err(|error| Error::invalid_argument(format!("Invalid session status: {error}")))
}

fn model_to_domain(model: agent_session::Model) -> Result<AgentSession> {
    let agent_type = parse_agent_type(&model.agent_type)?;
    let status = parse_status(&model.status)?;

    if model.model.trim().is_empty() {
        return Err(Error::invalid_argument(
            "Invalid persisted session schema: `model` must be non-empty",
        ));
    }

    Ok(AgentSession {
        id: model.id,
        session_summary_id: model.session_summary_id,
        agent_type,
        model: model.model,
        parent_session_id: model.parent_session_id,
        started_at: model.started_at,
        ended_at: model.ended_at,
        duration_ms: model.duration_ms,
        status,
        prompt_summary: model.prompt_summary,
        result_summary: model.result_summary,
        token_count: model.token_count,
        tool_calls_count: model.tool_calls_count,
        delegations_count: model.delegations_count,
        project_id: model.project_id,
        worktree_id: model.worktree_id,
    })
}

#[async_trait]
impl AgentSessionRepository for SeaOrmAgentSessionRepository {
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        validate_session_schema(session)?;
        let active: agent_session::ActiveModel = session.clone().into();
        active.insert(self.db.as_ref()).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        let model = agent_session::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(map_db_err)?;

        model.map(model_to_domain).transpose()
    }

    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        validate_session_schema(session)?;
        let active: agent_session::ActiveModel = session.clone().into();
        active.update(self.db.as_ref()).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let mut condition = Condition::all();

        if let Some(session_summary_id) = query.session_summary_id {
            condition =
                condition.add(agent_session::Column::SessionSummaryId.eq(session_summary_id));
        }
        if let Some(parent_session_id) = query.parent_session_id {
            condition = condition.add(agent_session::Column::ParentSessionId.eq(parent_session_id));
        }
        if let Some(agent_type) = query.agent_type {
            condition = condition.add(agent_session::Column::AgentType.eq(agent_type.as_str()));
        }
        if let Some(status) = query.status {
            condition = condition.add(agent_session::Column::Status.eq(status.as_str()));
        }
        if let Some(project_id) = query.project_id {
            condition = condition.add(agent_session::Column::ProjectId.eq(project_id));
        }
        if let Some(worktree_id) = query.worktree_id {
            condition = condition.add(agent_session::Column::WorktreeId.eq(worktree_id));
        }

        let mut select = agent_session::Entity::find()
            .filter(condition)
            .order_by_desc(agent_session::Column::StartedAt);

        if let Some(limit) = query.limit {
            select = select.limit(limit as u64);
        }

        let models = select.all(self.db.as_ref()).await.map_err(map_db_err)?;
        models
            .into_iter()
            .map(model_to_domain)
            .collect::<Result<Vec<_>>>()
    }

    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        let models = agent_session::Entity::find()
            .filter(agent_session::Column::ProjectId.eq(project_id))
            .order_by_desc(agent_session::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(map_db_err)?;

        models
            .into_iter()
            .map(model_to_domain)
            .collect::<Result<Vec<_>>>()
    }

    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        let models = agent_session::Entity::find()
            .filter(agent_session::Column::WorktreeId.eq(worktree_id))
            .order_by_desc(agent_session::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(map_db_err)?;

        models
            .into_iter()
            .map(model_to_domain)
            .collect::<Result<Vec<_>>>()
    }
}
