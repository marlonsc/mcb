#![allow(missing_docs)]

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, Delegation, ToolCall,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    AgentCheckpointRepository, AgentEventRepository, AgentSessionQuery, AgentSessionRepository,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::constants::database::{FALLBACK_AGENT_MODEL, FALLBACK_AGENT_PROMPT};
use crate::database::seaorm::entities::{
    agent_session, checkpoint, delegation, organization, project, tool_call,
};

pub struct SeaOrmAgentRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAgentRepository {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn db_err<E>(context: &str, source: E) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::memory_with_source(context, source)
    }

    async fn ensure_org_exists(&self, timestamp: i64) -> Result<()> {
        let existing = organization::Entity::find_by_id(DEFAULT_ORG_ID)
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("check existing org", e))?;

        if existing.is_some() {
            return Ok(());
        }

        let org = organization::ActiveModel {
            id: Set(DEFAULT_ORG_ID.to_owned()),
            name: Set(DEFAULT_ORG_NAME.to_owned()),
            slug: Set(DEFAULT_ORG_NAME.to_lowercase()),
            settings_json: Set("{}".to_owned()),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
        };

        organization::Entity::insert(org)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("auto-create default org", e))?;

        Ok(())
    }

    async fn ensure_project_exists(&self, project_id: &str, timestamp: i64) -> Result<()> {
        self.ensure_org_exists(timestamp).await?;

        let existing = project::Entity::find_by_id(project_id.to_owned())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("check existing project", e))?;

        if existing.is_some() {
            return Ok(());
        }

        let row = project::ActiveModel {
            id: Set(project_id.to_owned()),
            org_id: Set(DEFAULT_ORG_ID.to_owned()),
            name: Set(format!("Project {project_id}")),
            path: Set(project_id.to_owned()),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
        };

        project::Entity::insert(row)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("auto-create project", e))?;

        Ok(())
    }

    async fn ensure_session_exists(&self, session_id: &str, timestamp: i64) -> Result<()> {
        let existing = agent_session::Entity::find_by_id(session_id.to_owned())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("check existing session", e))?;

        if existing.is_some() {
            return Ok(());
        }

        self.ensure_org_exists(timestamp).await?;

        let fallback = AgentSession {
            id: session_id.to_owned(),
            session_summary_id: format!("auto-summary-{session_id}"),
            agent_type: AgentType::Sisyphus,
            model: FALLBACK_AGENT_MODEL.to_owned(),
            parent_session_id: None,
            started_at: timestamp,
            ended_at: None,
            duration_ms: None,
            status: AgentSessionStatus::Active,
            prompt_summary: Some(FALLBACK_AGENT_PROMPT.to_owned()),
            result_summary: None,
            token_count: None,
            tool_calls_count: None,
            delegations_count: None,
            project_id: None,
            worktree_id: None,
        };

        let active: agent_session::ActiveModel = fallback.into();
        agent_session::Entity::insert(active)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("auto-create agent session", e))?;

        Ok(())
    }
}

#[async_trait]
impl AgentSessionRepository for SeaOrmAgentRepository {
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        if let Some(project_id) = &session.project_id {
            self.ensure_project_exists(project_id, session.started_at)
                .await?;
        } else {
            self.ensure_org_exists(session.started_at).await?;
        }

        if let Some(parent_session_id) = &session.parent_session_id {
            self.ensure_session_exists(parent_session_id, session.started_at)
                .await?;
        }

        let active: agent_session::ActiveModel = session.clone().into();

        agent_session::Entity::insert(active)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("create agent session", e))?;

        Ok(())
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        let session = agent_session::Entity::find_by_id(id.to_owned())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get agent session", e))?;

        Ok(session.map(Into::into))
    }

    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        if let Some(project_id) = &session.project_id {
            self.ensure_project_exists(project_id, session.started_at)
                .await?;
        }

        if let Some(parent_session_id) = &session.parent_session_id {
            self.ensure_session_exists(parent_session_id, session.started_at)
                .await?;
        }

        let active: agent_session::ActiveModel = session.clone().into();
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("update agent session", e))?;

        Ok(())
    }

    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let mut q = agent_session::Entity::find();

        if let Some(session_summary_id) = query.session_summary_id {
            q = q.filter(agent_session::Column::SessionSummaryId.eq(session_summary_id));
        }
        if let Some(parent_session_id) = query.parent_session_id {
            q = q.filter(agent_session::Column::ParentSessionId.eq(parent_session_id));
        }
        if let Some(agent_type) = query.agent_type {
            q = q.filter(agent_session::Column::AgentType.eq(agent_type.to_string()));
        }
        if let Some(status) = query.status {
            q = q.filter(agent_session::Column::Status.eq(status.to_string()));
        }
        if let Some(project_id) = query.project_id {
            q = q.filter(agent_session::Column::ProjectId.eq(project_id));
        }
        if let Some(worktree_id) = query.worktree_id {
            q = q.filter(agent_session::Column::WorktreeId.eq(worktree_id));
        }

        q = q.order_by_desc(agent_session::Column::StartedAt);

        if let Some(limit) = query.limit {
            q = q.limit(limit as u64);
        }

        let sessions = q
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("list agent sessions", e))?;

        Ok(sessions.into_iter().map(Into::into).collect())
    }

    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        let sessions = agent_session::Entity::find()
            .filter(agent_session::Column::ProjectId.eq(project_id.to_owned()))
            .order_by_desc(agent_session::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("list project sessions", e))?;

        Ok(sessions.into_iter().map(Into::into).collect())
    }

    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        let sessions = agent_session::Entity::find()
            .filter(agent_session::Column::WorktreeId.eq(worktree_id.to_owned()))
            .order_by_desc(agent_session::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("list worktree sessions", e))?;

        Ok(sessions.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl AgentEventRepository for SeaOrmAgentRepository {
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()> {
        self.ensure_session_exists(&delegation.parent_session_id, delegation.created_at)
            .await?;
        self.ensure_session_exists(&delegation.child_session_id, delegation.created_at)
            .await?;

        let active: delegation::ActiveModel = delegation.clone().into();
        delegation::Entity::insert(active)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("store delegation", e))?;

        Ok(())
    }

    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()> {
        self.ensure_session_exists(&tool_call.session_id, tool_call.created_at)
            .await?;

        let parent_session = agent_session::Entity::find_by_id(tool_call.session_id.clone())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("load session for tool call", e))?;

        let mut active: tool_call::ActiveModel = tool_call.clone().into();
        active.org_id = Set(Some(DEFAULT_ORG_ID.to_owned()));
        active.project_id = Set(parent_session.and_then(|s| s.project_id));
        active.repo_id = Set(None);

        tool_call::Entity::insert(active)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("store tool call", e))?;

        Ok(())
    }
}

#[async_trait]
impl AgentCheckpointRepository for SeaOrmAgentRepository {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.ensure_session_exists(&checkpoint.session_id, checkpoint.created_at)
            .await?;

        let active: checkpoint::ActiveModel = checkpoint.clone().into();
        checkpoint::Entity::insert(active)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("store checkpoint", e))?;

        Ok(())
    }

    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let checkpoint = checkpoint::Entity::find_by_id(id.to_owned())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get checkpoint", e))?;

        Ok(checkpoint.map(Into::into))
    }

    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.ensure_session_exists(&checkpoint.session_id, checkpoint.created_at)
            .await?;

        let active: checkpoint::ActiveModel = checkpoint.clone().into();
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("update checkpoint", e))?;

        Ok(())
    }
}
