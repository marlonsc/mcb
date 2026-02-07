//! Agent Session Service Use Case

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall,
};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::agent_repository::{AgentRepository, AgentSessionQuery};
use mcb_domain::ports::services::AgentSessionServiceInterface;

pub struct AgentSessionServiceImpl {
    repository: Arc<dyn AgentRepository>,
}

impl AgentSessionServiceImpl {
    pub fn new(repository: Arc<dyn AgentRepository>) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

#[async_trait]
impl AgentSessionServiceInterface for AgentSessionServiceImpl {
    async fn create_session(&self, session: AgentSession) -> Result<String> {
        let id = session.id.clone();
        self.repository.create_session(&session).await?;
        Ok(id)
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        self.repository.get_session(id).await
    }

    async fn update_session(&self, session: AgentSession) -> Result<()> {
        self.repository.update_session(&session).await
    }

    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        self.repository.list_sessions(query).await
    }

    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()> {
        let session = self.repository.get_session(id).await?;
        if let Some(mut session) = session {
            let now = Self::current_timestamp();
            session.ended_at = Some(now);
            session.duration_ms = Some((now - session.started_at) * 1000);
            session.status = status;
            session.result_summary = result_summary;
            self.repository.update_session(&session).await?;
        }
        Ok(())
    }

    async fn store_delegation(&self, delegation: Delegation) -> Result<String> {
        let id = delegation.id.clone();
        self.repository.store_delegation(&delegation).await?;
        Ok(id)
    }

    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String> {
        let id = tool_call.id.clone();
        self.repository.store_tool_call(&tool_call).await?;
        Ok(id)
    }

    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String> {
        let id = checkpoint.id.clone();
        self.repository.store_checkpoint(&checkpoint).await?;
        Ok(id)
    }

    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        self.repository.get_checkpoint(id).await
    }

    async fn restore_checkpoint(&self, id: &str) -> Result<()> {
        let checkpoint = self.repository.get_checkpoint(id).await?;
        if let Some(mut checkpoint) = checkpoint {
            checkpoint.restored_at = Some(Self::current_timestamp());
            self.repository.update_checkpoint(&checkpoint).await?;
        }
        Ok(())
    }
}
