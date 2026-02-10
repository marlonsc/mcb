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

/// Application service for managing agent session lifecycle and persistence.
///
/// Implements the `AgentSessionServiceInterface` to provide session creation, retrieval,
/// updates, and termination. Delegates all persistence operations to the injected
/// `AgentRepository`, enabling clean separation between business logic and data access.
/// Also manages session-related artifacts like delegations, tool calls, and checkpoints.
pub struct AgentSessionServiceImpl {
    repository: Arc<dyn AgentRepository>,
}

impl AgentSessionServiceImpl {
    /// Initializes the service with the required agent repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository implementation for persisting and retrieving agent sessions.
    ///   This is typically injected via dependency injection and may be backed by SQLite,
    ///   PostgreSQL, or another persistent storage mechanism.
    ///
    /// # Returns
    ///
    /// A new `AgentSessionServiceImpl` instance ready to manage agent sessions.
    pub fn new(repository: Arc<dyn AgentRepository>) -> Self {
        Self { repository }
    }

    /// Returns the current Unix timestamp in seconds.
    ///
    /// Used throughout the service to record session start times, end times, and
    /// checkpoint restoration timestamps. Falls back to 0 if the system clock is
    /// unavailable (which should be extremely rare).
    ///
    /// # Returns
    ///
    /// Current Unix timestamp as seconds since UNIX_EPOCH, or 0 if unavailable.
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
