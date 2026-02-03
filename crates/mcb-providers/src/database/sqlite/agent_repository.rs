//! SQLite agent repository using the domain port [`DatabaseExecutor`].
//!
//! Implements [`AgentRepository`] via [`DatabaseExecutor`]; no direct sqlx in this module.

use super::row_convert;

use async_trait::async_trait;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, Delegation, ToolCall};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::agent_repository::{AgentRepository, AgentSessionQuery};
use std::sync::Arc;
use tracing::debug;

/// SQLite-based agent repository using the database executor port.
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
impl AgentRepository for SqliteAgentRepository {
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        let params = [
            SqlParam::String(session.id.clone()),
            SqlParam::String(session.session_summary_id.clone()),
            SqlParam::String(session.agent_type.as_str().to_string()),
            SqlParam::String(session.model.clone()),
            session
                .parent_session_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::I64(session.started_at),
            session.ended_at.map_or(SqlParam::Null, SqlParam::I64),
            session.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::String(session.status.as_str().to_string()),
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
                    delegations_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Stored agent session: {}", session.id);
        Ok(())
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM agent_sessions WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_agent_session(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode agent session row", e))?,
            )),
            None => Ok(None),
        }
    }

    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        let params = [
            SqlParam::String(session.session_summary_id.clone()),
            SqlParam::String(session.agent_type.as_str().to_string()),
            SqlParam::String(session.model.clone()),
            session
                .parent_session_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::I64(session.started_at),
            session.ended_at.map_or(SqlParam::Null, SqlParam::I64),
            session.duration_ms.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::String(session.status.as_str().to_string()),
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
                    delegations_count = ?
                WHERE id = ?
                ",
                &params,
            )
            .await?;

        debug!("Updated agent session: {}", session.id);
        Ok(())
    }

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
            params.push(SqlParam::String(agent_type.as_str().to_string()));
        }
        if let Some(status) = &query.status {
            sql.push_str(" AND status = ?");
            params.push(SqlParam::String(status.as_str().to_string()));
        }

        sql.push_str(" ORDER BY started_at DESC");
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(SqlParam::I64(limit as i64));
        }

        let rows = self.executor.query_all(&sql, &params).await?;
        let mut sessions = Vec::with_capacity(rows.len());
        for row in rows {
            sessions.push(
                row_convert::row_to_agent_session(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode agent session row", e))?,
            );
        }
        Ok(sessions)
    }

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

    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let snapshot_json = serde_json::to_string(&checkpoint.snapshot_data)
            .map_err(|e| Error::memory_with_source("serialize checkpoint snapshot", e))?;

        let params = [
            SqlParam::String(checkpoint.id.clone()),
            SqlParam::String(checkpoint.session_id.clone()),
            SqlParam::String(checkpoint.checkpoint_type.as_str().to_string()),
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

    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM checkpoints WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_checkpoint(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode checkpoint row", e))?,
            )),
            None => Ok(None),
        }
    }

    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let snapshot_json = serde_json::to_string(&checkpoint.snapshot_data)
            .map_err(|e| Error::memory_with_source("serialize checkpoint snapshot", e))?;

        let params = [
            SqlParam::String(checkpoint.session_id.clone()),
            SqlParam::String(checkpoint.checkpoint_type.as_str().to_string()),
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
