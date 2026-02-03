//! Agent session repository port.

use crate::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, Delegation, ToolCall,
};
use crate::error::Result;
use async_trait::async_trait;

/// Query filters for agent session lookups.
#[derive(Debug, Clone, Default)]
pub struct AgentSessionQuery {
    pub session_summary_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub agent_type: Option<AgentType>,
    pub status: Option<AgentSessionStatus>,
    pub limit: Option<usize>,
}

/// Port for agent session persistence.
#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn create_session(&self, session: &AgentSession) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    async fn update_session(&self, session: &AgentSession) -> Result<()>;
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;

    async fn store_delegation(&self, delegation: &Delegation) -> Result<()>;
    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()>;

    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
}
