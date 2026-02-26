//! Agent Session Service Use Case
//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#use-cases)
//!
//! # Overview
//! The `AgentSessionService` handles the lifecycle and persistence of autonomous agent sessions.
//! It serves as the system of record for agent interactions, state transitions, and execution history.
//!
//! # Responsibilities
//! - **Session Lifecycle**: Creating, updating, and terminating sessions with proper status tracking.
//! - **Artifact Management**: Storing delegations (sub-tasks), tool calls, and checkpoints.
//! - **State Persistence**: Ensuring session data is reliably saved to the underlying repository.
//! - **Querying**: Providing flexible access to session history by project, worktree, or status.
//!
//! # Architecture
//! Implements `AgentSessionServiceInterface` and delegates data access to `AgentRepository`.
//! It acts as a facade for session-related operations, abstracting the storage details
//! from the application layer.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall,
};
use mcb_domain::error::Result;
use mcb_domain::ports::{
    AgentRepository, AgentSessionManager, AgentSessionQuery, CheckpointManager, DelegationTracker,
};
use mcb_domain::utils::time as domain_time;

/// Application service for managing agent session lifecycle and persistence.
///
/// Implements the `AgentSessionServiceInterface` to provide robust session management,
/// including creation, state transitions, and historical querying. It acts as the
/// authoritative source for agent execution data, coordinating with the `AgentRepository`
/// for durable storage of sessions, tool calls, and checkpoints.
pub struct AgentSessionServiceImpl {
    repository: Arc<dyn AgentRepository>,
}

impl AgentSessionServiceImpl {
    /// Initializes the service with the required agent repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository implementation for persisting and retrieving agent sessions.
    ///   This is typically injected via dependency injection and may be backed by `SQLite`,
    ///   `PostgreSQL`, or another persistent storage mechanism.
    ///
    /// # Returns
    ///
    /// A new `AgentSessionServiceImpl` instance ready to manage agent sessions.
    pub fn new(repository: Arc<dyn AgentRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
#[async_trait]
impl AgentSessionManager for AgentSessionServiceImpl {
    /// # Errors
    ///
    /// Returns an error if the repository fails to persist the session.
    async fn create_session(&self, session: AgentSession) -> Result<String> {
        let id = session.id.clone();
        self.repository.create_session(&session).await?;
        Ok(id)
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        self.repository.get_session(id).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository fails to update the session.
    async fn update_session(&self, session: AgentSession) -> Result<()> {
        self.repository.update_session(&session).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        self.repository.list_sessions(query).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        self.repository.list_sessions_by_project(project_id).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        self.repository.list_sessions_by_worktree(worktree_id).await
    }

    /// # Errors
    ///
    /// Returns an error if the session lookup or update fails.
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()> {
        let session = self.repository.get_session(id).await?;
        if let Some(mut session) = session {
            let now = domain_time::epoch_secs_i64()?;
            session.ended_at = Some(now);
            session.duration_ms = Some((now - session.started_at) * 1000);
            session.status = status;
            session.result_summary = result_summary;
            self.repository.update_session(&session).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl DelegationTracker for AgentSessionServiceImpl {
    /// # Errors
    ///
    /// Returns an error if the repository fails to persist the delegation.
    async fn store_delegation(&self, delegation: Delegation) -> Result<String> {
        let id = delegation.id.clone();
        self.repository.store_delegation(&delegation).await?;
        Ok(id)
    }

    /// # Errors
    ///
    /// Returns an error if the repository fails to persist the tool call.
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String> {
        let id = tool_call.id.clone();
        self.repository.store_tool_call(&tool_call).await?;
        Ok(id)
    }
}

#[async_trait]
impl CheckpointManager for AgentSessionServiceImpl {
    /// # Errors
    ///
    /// Returns an error if the repository fails to persist the checkpoint.
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String> {
        let id = checkpoint.id.clone();
        self.repository.store_checkpoint(&checkpoint).await?;
        Ok(id)
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        self.repository.get_checkpoint(id).await
    }

    /// # Errors
    ///
    /// Returns an error if the checkpoint lookup or update fails.
    async fn restore_checkpoint(&self, id: &str) -> Result<()> {
        let checkpoint = self.repository.get_checkpoint(id).await?;
        if let Some(mut checkpoint) = checkpoint {
            checkpoint.restored_at = Some(domain_time::epoch_secs_i64()?);
            self.repository.update_checkpoint(&checkpoint).await?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::services::{
    AGENT_SESSION_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
};

fn build_agent_session_service_from_registry(
    context: &dyn std::any::Any,
) -> mcb_domain::error::Result<Arc<dyn mcb_domain::ports::AgentSessionServiceInterface>> {
    let ctx = context
        .downcast_ref::<crate::resolution_context::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Agent session service builder requires ServiceResolutionContext",
            )
        })?;

    let repos = mcb_domain::registry::database::resolve_database_repositories(
        "seaorm",
        Box::new(ctx.db.clone()),
        "default".to_owned(),
    )
    .map_err(mcb_domain::error::Error::internal)?;

    Ok(Arc::new(AgentSessionServiceImpl::new(repos.agent)))
}

#[linkme::distributed_slice(SERVICES_REGISTRY)]
static AGENT_SESSION_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: AGENT_SESSION_SERVICE_NAME,
    build: ServiceBuilder::AgentSession(build_agent_session_service_from_registry),
};
