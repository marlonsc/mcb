//! SQLite Agent Repository
//!
//! # Overview
//! The `SqliteAgentRepository` manages the persistence of agent sessions and their related artifacts
//! (tool calls, delegations, checkpoints). It enables long-running, stateful agent interactions
//! by reliably storing execution history in a relational database.
//!
//! # Responsibilities
//! - **Session Management**: CRUD operations for `AgentSession` entities.
//! - **Audit Trail**: Recording all tool calls and delegations for debugging and analysis.
//! - **State Recovery**: Managing `Checkpoint` storage to allow sessions to resume or rollback.
//! - **Querying**: Filtering sessions by project, status, or hierarchy.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, Delegation, ToolCall};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::agent_repository::{
    AgentCheckpointRepository, AgentEventRepository, AgentSessionQuery, AgentSessionRepository,
};
use mcb_domain::utils::mask_id;
use tracing::debug;

use super::query_helpers;
use super::row_convert;

/// SQLite-based implementation of the `AgentRepository`.
///
/// Implements data access patterns for agent sessions, delegations, tool calls, and checkpoints.
/// Ensures referential integrity for project/organization links and handles JSON serialization
/// for complex session state artifacts.
pub struct SqliteAgentRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteAgentRepository {
    /// Create a repository that uses the given executor (from provider factory).
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
/// Persistent agent session repository using SQLite.
impl AgentSessionRepository for SqliteAgentRepository {
    /// Creates a new agent session.
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        if let Some(project_id) = &session.project_id {
            super::ensure_parent::ensure_org_and_project(
                self.executor.as_ref(),
                project_id,
                session.started_at,
            )
            .await?;
        } else {
            super::ensure_parent::ensure_org_exists(self.executor.as_ref(), session.started_at)
                .await?;
        }

        let params = [
            SqlParam::String(session.id.clone()),
            SqlParam::String(session.session_summary_id.clone()),
            SqlParam::String(session.agent_type.as_str().to_owned()),
            SqlParam::String(session.model.clone()),
            session
                .parent_session_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::I64(session.started_at),
            session.ended_at.map_or(SqlParam::Null, SqlParam::I64),
            session.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::String(session.status.as_str().to_owned()),
            session
                .prompt_summary
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session
                .result_summary
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session.token_count.map_or(SqlParam::Null, SqlParam::I64),
            session
                .tool_calls_count
                .map_or(SqlParam::Null, SqlParam::I64),
            session
                .delegations_count
                .map_or(SqlParam::Null, SqlParam::I64),
            session
                .project_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session
                .worktree_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO agent_sessions (
                    id,
                    session_summary_id,
                    agent_type,
                    model,
                    parent_session_id,
                    started_at,
                    ended_at,
                    duration_ms,
                    status,
                    prompt_summary,
                    result_summary,
                    token_count,
                    tool_calls_count,
                    delegations_count,
                    project_id,
                    worktree_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Stored agent session: {}", mask_id(session.id.as_str()));
        Ok(())
    }

    /// Retrieves a session by ID.
    // TODO(qlty): Found 17 lines of similar code in 3 locations (mass = 91)
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM agent_sessions WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_convert::row_to_agent_session,
        )
        .await
    }

    /// Updates an existing session.
    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        let params = [
            SqlParam::String(session.session_summary_id.clone()),
            SqlParam::String(session.agent_type.as_str().to_owned()),
            SqlParam::String(session.model.clone()),
            session
                .parent_session_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::I64(session.started_at),
            session.ended_at.map_or(SqlParam::Null, SqlParam::I64),
            session.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::String(session.status.as_str().to_owned()),
            session
                .prompt_summary
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session
                .result_summary
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session.token_count.map_or(SqlParam::Null, SqlParam::I64),
            session
                .tool_calls_count
                .map_or(SqlParam::Null, SqlParam::I64),
            session
                .delegations_count
                .map_or(SqlParam::Null, SqlParam::I64),
            session
                .project_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            session
                .worktree_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(session.id.clone()),
        ];

        self.executor
            .execute(
                r"
                UPDATE agent_sessions
                SET session_summary_id = ?,
                    agent_type = ?,
                    model = ?,
                    parent_session_id = ?,
                    started_at = ?,
                    ended_at = ?,
                    duration_ms = ?,
                    status = ?,
                    prompt_summary = ?,
                    result_summary = ?,
                    token_count = ?,
                    tool_calls_count = ?,
                    delegations_count = ?,
                    project_id = ?,
                    worktree_id = ?
                WHERE id = ?
                ",
                &params,
            )
            .await?;

        debug!("Updated agent session: {}", mask_id(session.id.as_str()));
        Ok(())
    }

    /// Lists sessions matching the query criteria.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let mut sql = String::from("SELECT * FROM agent_sessions WHERE 1=1");
        let mut params: Vec<SqlParam> = Vec::new();

        if let Some(session_summary_id) = &query.session_summary_id {
            sql.push_str(" AND session_summary_id = ?");
            params.push(SqlParam::String(session_summary_id.clone()));
        }
        if let Some(parent_session_id) = &query.parent_session_id {
            sql.push_str(" AND parent_session_id = ?");
            params.push(SqlParam::String(parent_session_id.clone()));
        }
        if let Some(agent_type) = &query.agent_type {
            sql.push_str(" AND agent_type = ?");
            params.push(SqlParam::String(agent_type.as_str().to_owned()));
        }
        if let Some(status) = &query.status {
            sql.push_str(" AND status = ?");
            params.push(SqlParam::String(status.as_str().to_owned()));
        }
        if let Some(project_id) = &query.project_id {
            sql.push_str(" AND project_id = ?");
            params.push(SqlParam::String(project_id.clone()));
        }
        if let Some(worktree_id) = &query.worktree_id {
            sql.push_str(" AND worktree_id = ?");
            params.push(SqlParam::String(worktree_id.clone()));
        }

        sql.push_str(" ORDER BY started_at DESC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(SqlParam::I64(limit as i64));
        }

        query_helpers::query_all(
            &self.executor,
            &sql,
            &params,
            row_convert::row_to_agent_session,
            "agent session",
        )
        .await
    }

    /// Lists sessions for a specific project.
    // TODO(qlty): Found 18 lines of similar code in 3 locations (mass = 97)
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM agent_sessions WHERE project_id = ? ORDER BY started_at DESC",
            &[SqlParam::String(project_id.to_owned())],
            row_convert::row_to_agent_session,
            "agent session",
        )
        .await
    }

    /// Lists sessions for a specific worktree.
    // TODO(qlty): Found 18 lines of similar code in 3 locations (mass = 97)
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM agent_sessions WHERE worktree_id = ? ORDER BY started_at DESC",
            &[SqlParam::String(worktree_id.to_owned())],
            row_convert::row_to_agent_session,
            "agent session",
        )
        .await
    }
}

#[async_trait]
/// Persistent agent event repository using SQLite.
impl AgentEventRepository for SqliteAgentRepository {
    /// Stores a delegation record.
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()> {
        let params = [
            SqlParam::String(delegation.id.clone()),
            SqlParam::String(delegation.parent_session_id.clone()),
            SqlParam::String(delegation.child_session_id.clone()),
            SqlParam::String(delegation.prompt.clone()),
            delegation
                .prompt_embedding_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            delegation
                .result
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::Bool(delegation.success),
            SqlParam::I64(delegation.created_at),
            delegation
                .completed_at
                .map_or(SqlParam::Null, SqlParam::I64),
            delegation.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO delegations (
                    id,
                    parent_session_id,
                    child_session_id,
                    prompt,
                    prompt_embedding_id,
                    result,
                    success,
                    created_at,
                    completed_at,
                    duration_ms
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Stored delegation: {}", delegation.id);
        Ok(())
    }

    /// Stores a tool call record.
    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()> {
        let params = [
            SqlParam::String(tool_call.id.clone()),
            SqlParam::String(tool_call.session_id.clone()),
            SqlParam::String(tool_call.tool_name.clone()),
            tool_call
                .params_summary
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::Bool(tool_call.success),
            tool_call
                .error_message
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            tool_call.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::I64(tool_call.created_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO tool_calls (
                    id,
                    session_id,
                    tool_name,
                    params_summary,
                    success,
                    error_message,
                    duration_ms,
                    created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Stored tool call: {}", tool_call.id);
        Ok(())
    }
}

#[async_trait]
/// Persistent agent checkpoint repository using SQLite.
impl AgentCheckpointRepository for SqliteAgentRepository {
    /// Stores a session checkpoint.
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let snapshot_json = serde_json::to_string(&checkpoint.snapshot_data)
            .map_err(|e| Error::memory_with_source("serialize checkpoint snapshot", e))?;

        let params = [
            SqlParam::String(checkpoint.id.clone()),
            SqlParam::String(checkpoint.session_id.clone()),
            SqlParam::String(checkpoint.checkpoint_type.as_str().to_owned()),
            SqlParam::String(checkpoint.description.clone()),
            SqlParam::String(snapshot_json),
            SqlParam::I64(checkpoint.created_at),
            checkpoint.restored_at.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::Bool(checkpoint.expired),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO checkpoints (
                    id,
                    session_id,
                    checkpoint_type,
                    description,
                    snapshot_data,
                    created_at,
                    restored_at,
                    expired
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Stored checkpoint: {}", checkpoint.id);
        Ok(())
    }

    /// Retrieves a checkpoint by ID.
    // TODO(qlty): Found 17 lines of similar code in 3 locations (mass = 91)
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM checkpoints WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_convert::row_to_checkpoint,
        )
        .await
    }

    /// Updates an existing checkpoint.
    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let snapshot_json = serde_json::to_string(&checkpoint.snapshot_data)
            .map_err(|e| Error::memory_with_source("serialize checkpoint snapshot", e))?;

        let params = [
            SqlParam::String(checkpoint.session_id.clone()),
            SqlParam::String(checkpoint.checkpoint_type.as_str().to_owned()),
            SqlParam::String(checkpoint.description.clone()),
            SqlParam::String(snapshot_json),
            SqlParam::I64(checkpoint.created_at),
            checkpoint.restored_at.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::Bool(checkpoint.expired),
            SqlParam::String(checkpoint.id.clone()),
        ];

        self.executor
            .execute(
                r"
                UPDATE checkpoints
                SET session_id = ?,
                    checkpoint_type = ?,
                    description = ?,
                    snapshot_data = ?,
                    created_at = ?,
                    restored_at = ?,
                    expired = ?
                WHERE id = ?
                ",
                &params,
            )
            .await?;

        debug!("Updated checkpoint: {}", checkpoint.id);
        Ok(())
    }
}
