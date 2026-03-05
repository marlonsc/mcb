//! Agent session service ports.

use async_trait::async_trait;

use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::error::Result;
use crate::ports::repositories::agent::AgentSessionQuery;

/// Manages agent session lifecycle.
#[async_trait]
pub trait AgentSessionManager: Send + Sync {
    /// Create a new agent session.
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    /// Get an agent session by ID.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    /// Update an existing agent session.
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    /// List agent sessions matching query filters.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    /// List agent sessions associated with a project.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;
    /// List agent sessions associated with a worktree.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
    /// Mark a session as ended with final status.
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
}

/// Tracks delegations and tool calls.
#[async_trait]
pub trait DelegationTracker: Send + Sync {
    /// Store a delegation record.
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    /// Store a tool call record.
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
}

/// Manages checkpoints and restoration.
#[async_trait]
pub trait CheckpointManager: Send + Sync {
    /// Store an agent checkpoint.
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    /// Get a checkpoint by its ID.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Restore an agent state from a checkpoint.
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}

/// Aggregate trait for agent session service.
pub trait AgentSessionServiceInterface:
    AgentSessionManager + DelegationTracker + CheckpointManager + Send + Sync
{
}

impl<T> AgentSessionServiceInterface for T where
    T: AgentSessionManager + DelegationTracker + CheckpointManager + Send + Sync
{
}
