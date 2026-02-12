use async_trait::async_trait;

use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::error::Result;
use crate::ports::repositories::agent_repository::AgentSessionQuery;

/// Port for agent session lifecycle and delegation tracking (create, list, end sessions).
#[async_trait]
pub trait AgentSessionServiceInterface: Send + Sync {
    /// Create a new agent session
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    /// Get an agent session by its ID
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    /// Update an agent session
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    /// List all agent sessions
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    /// List all agent sessions by project ID
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;
    /// List all agent sessions by worktree ID
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
    /// Store a delegation
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    /// Store a tool call
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
    /// Store a checkpoint
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    /// Get a checkpoint by its ID
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Restore a checkpoint
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}
