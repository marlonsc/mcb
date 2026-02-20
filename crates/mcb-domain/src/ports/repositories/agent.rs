//! Agent Repository Port
//!
//! # Overview
//! Defines the interface for persisting agent execution state, including sessions,
//! delegations, tool calls, and checkpoints.

use async_trait::async_trait;

use crate::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, Delegation, ToolCall,
};
use crate::error::Result;

/// Query filters for agent session lookups.
///
/// Provides flexible filtering options for retrieving agent sessions from the repository.
/// All fields are optional - omitted fields are not used as filters.
#[derive(Debug, Clone, Default)]
pub struct AgentSessionQuery {
    /// Filter by session summary ID
    pub session_summary_id: Option<String>,
    /// Filter by parent session ID
    pub parent_session_id: Option<String>,
    /// Filter by agent type
    pub agent_type: Option<AgentType>,
    /// Filter by session status
    pub status: Option<AgentSessionStatus>,
    /// Filter by project ID
    pub project_id: Option<String>,
    /// Filter by worktree ID
    pub worktree_id: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Port for agent session persistence.
#[async_trait]
pub trait AgentSessionRepository: Send + Sync {
    /// Creates a new agent session.
    async fn create_session(&self, session: &AgentSession) -> Result<()>;

    /// Retrieves an agent session by ID.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;

    /// Updates an existing agent session.
    async fn update_session(&self, session: &AgentSession) -> Result<()>;

    /// Lists agent sessions matching the provided query filters.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;

    /// Lists agent sessions belonging to a specific project.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;

    /// Lists agent sessions associated with a specific worktree.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
}

/// Port for agent event persistence (delegations, tool calls).
#[async_trait]
pub trait AgentEventRepository: Send + Sync {
    /// Stores a delegation record.
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()>;

    /// Stores a tool call record.
    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()>;
}

/// Port for agent checkpoint persistence.
#[async_trait]
pub trait AgentCheckpointRepository: Send + Sync {
    /// Stores a checkpoint.
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;

    /// Retrieves a checkpoint by ID.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;

    /// Updates an existing checkpoint.
    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
}

/// Aggregate trait for full agent persistence capabilities.
///
/// This trait combines session, event, and checkpoint management.
/// It is automatically implemented for any type that implements the sub-traits.
pub trait AgentRepository:
    AgentSessionRepository + AgentEventRepository + AgentCheckpointRepository + Send + Sync
{
}

impl<T> AgentRepository for T where
    T: AgentSessionRepository + AgentEventRepository + AgentCheckpointRepository + Send + Sync
{
}
