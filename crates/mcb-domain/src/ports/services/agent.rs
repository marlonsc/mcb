use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::error::Result;
use crate::ports::repositories::agent_repository::AgentSessionQuery;
use async_trait::async_trait;

/// Port for agent session lifecycle and delegation tracking (create, list, end sessions).
#[async_trait]
pub trait AgentSessionServiceInterface: Send + Sync {
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}
