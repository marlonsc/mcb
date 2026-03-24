//! SeaORM-backed agent repository (sessions, events, checkpoints).

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, Delegation, ToolCall};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    AgentCheckpointRepository, AgentEventRepository, AgentSessionQuery, AgentSessionRepository,
};
use mcb_utils::constants::values::{DEFAULT_ORG_ID, DEFAULT_SESSION_LIMIT};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use super::common::{db_error, ensure_org_and_project};
use crate::database::seaorm::entities::{agent_session, checkpoint, delegation, tool_call};

/// SeaORM-based agent repository.
pub struct SeaOrmAgentRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAgentRepository {
    /// Creates a new `SeaOrmAgentRepository`.
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    /// Verify that a referenced session exists (strict: no auto-creation).
    async fn require_session_exists(&self, session_id: &str) -> Result<()> {
        let existing = agent_session::Entity::find_by_id(session_id.to_owned())
            .one(self.db())
            .await
            .map_err(db_error("check existing session"))?;
        if existing.is_none() {
            return Err(Error::not_found(format!(
                "Agent session '{session_id}' not found. Sessions must be created before storing events."
            )));
        }
        Ok(())
    }

    /// List sessions filtered by a single column.
    async fn list_sessions_by_col(
        &self,
        col: agent_session::Column,
        value: &str,
        label: &str,
    ) -> Result<Vec<AgentSession>> {
        let sessions = agent_session::Entity::find()
            .filter(col.eq(value.to_owned()))
            .order_by_desc(agent_session::Column::StartedAt)
            .limit(DEFAULT_SESSION_LIMIT)
            .all(self.db())
            .await
            .map_err(db_error(label))?;
        Ok(sessions.into_iter().map(Into::into).collect())
    }

    /// Ensure parent prerequisites (org + optional project) exist.
    async fn ensure_session_prerequisites(&self, session: &AgentSession) -> Result<()> {
        let project_id = session.project_id.as_deref().unwrap_or(&session.id);
        ensure_org_and_project(self.db(), DEFAULT_ORG_ID, project_id, session.started_at).await
    }
}

#[async_trait]
impl AgentSessionRepository for SeaOrmAgentRepository {
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        self.ensure_session_prerequisites(session).await?;
        if let Some(parent_session_id) = &session.parent_session_id {
            self.require_session_exists(parent_session_id).await?;
        }
        sea_repo_insert!(self.db(), agent_session, session, "create agent session")
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        sea_repo_get_opt!(
            self.db(),
            agent_session,
            AgentSession,
            id,
            "get agent session"
        )
    }

    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        self.ensure_session_prerequisites(session).await?;
        if let Some(parent_session_id) = &session.parent_session_id {
            self.require_session_exists(parent_session_id).await?;
        }
        sea_repo_update!(self.db(), agent_session, session, "update agent session")
    }

    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let mut q = agent_session::Entity::find();
        if let Some(v) = query.session_summary_id {
            q = q.filter(agent_session::Column::SessionSummaryId.eq(v));
        }
        if let Some(v) = query.parent_session_id {
            q = q.filter(agent_session::Column::ParentSessionId.eq(v));
        }
        if let Some(v) = query.agent_type {
            q = q.filter(agent_session::Column::AgentType.eq(v.to_string()));
        }
        if let Some(v) = query.status {
            q = q.filter(agent_session::Column::Status.eq(v.to_string()));
        }
        if let Some(v) = query.project_id {
            q = q.filter(agent_session::Column::ProjectId.eq(v));
        }
        if let Some(v) = query.worktree_id {
            q = q.filter(agent_session::Column::WorktreeId.eq(v));
        }
        q = q
            .order_by_desc(agent_session::Column::StartedAt)
            .limit(query.limit.map_or(DEFAULT_SESSION_LIMIT, |l| l as u64));
        let sessions = q
            .all(self.db())
            .await
            .map_err(db_error("list agent sessions"))?;
        Ok(sessions.into_iter().map(Into::into).collect())
    }

    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        self.list_sessions_by_col(
            agent_session::Column::ProjectId,
            project_id,
            "list project sessions",
        )
        .await
    }

    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        self.list_sessions_by_col(
            agent_session::Column::WorktreeId,
            worktree_id,
            "list worktree sessions",
        )
        .await
    }
}

#[async_trait]
impl AgentEventRepository for SeaOrmAgentRepository {
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()> {
        self.require_session_exists(&delegation.parent_session_id)
            .await?;
        self.require_session_exists(&delegation.child_session_id)
            .await?;
        sea_repo_insert!(self.db(), delegation, delegation, "store delegation")
    }

    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()> {
        self.require_session_exists(&tool_call.session_id).await?;
        let parent_session = agent_session::Entity::find_by_id(tool_call.session_id.clone())
            .one(self.db())
            .await
            .map_err(db_error("load session for tool call"))?;
        let mut active: tool_call::ActiveModel = tool_call.clone().into();
        active.org_id = Set(Some(DEFAULT_ORG_ID.to_owned()));
        active.project_id = Set(parent_session.and_then(|s| s.project_id));
        active.repo_id = Set(None);
        tool_call::Entity::insert(active)
            .exec(self.db())
            .await
            .map_err(db_error("store tool call"))?;
        Ok(())
    }
}

#[async_trait]
impl AgentCheckpointRepository for SeaOrmAgentRepository {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.require_session_exists(&checkpoint.session_id).await?;
        sea_repo_insert!(self.db(), checkpoint, checkpoint, "store checkpoint")
    }

    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        sea_repo_get_opt!(self.db(), checkpoint, Checkpoint, id, "get checkpoint")
    }

    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.require_session_exists(&checkpoint.session_id).await?;
        sea_repo_update!(self.db(), checkpoint, checkpoint, "update checkpoint")
    }
}
