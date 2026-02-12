//! Provides agent domain definitions.
use async_trait::async_trait;

use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::error::Result;
use crate::ports::repositories::agent_repository::AgentSessionQuery;

/// Port for agent session lifecycle and delegation tracking (create, list, end sessions).
#[async_trait]
pub trait AgentSessionServiceInterface: Send + Sync {
    /// Performs the create session operation.
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    /// Performs the get session operation.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    /// Performs the update session operation.
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    /// Performs the list sessions operation.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    /// Performs the list sessions by project operation.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;
    /// Performs the list sessions by worktree operation.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
    /// Performs the end session operation.
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
    /// Performs the store delegation operation.
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    /// Performs the store tool call operation.
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
    /// Performs the store checkpoint operation.
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    /// Performs the get checkpoint operation.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Performs the restore checkpoint operation.
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}
