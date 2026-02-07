//! Agent session repository port.

use crate::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, Delegation, ToolCall,
};
use crate::error::Result;
use async_trait::async_trait;

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
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Port for agent session persistence.
///
/// Defines the interface for storing and retrieving agent sessions, delegations, tool calls,
/// and checkpoints. Implementations handle all persistence concerns for the agent domain.
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// Creates a new agent session in the repository.
    ///
    /// # Arguments
    /// * `session` - The agent session to create
    ///
    /// # Errors
    /// Returns an error if the session cannot be created (e.g., duplicate ID, storage failure).
    async fn create_session(&self, session: &AgentSession) -> Result<()>;

    /// Retrieves an agent session by ID.
    ///
    /// # Arguments
    /// * `id` - The session ID to retrieve
    ///
    /// # Returns
    /// `Ok(Some(session))` if found, `Ok(None)` if not found.
    ///
    /// # Errors
    /// Returns an error if the retrieval fails (e.g., storage error).
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;

    /// Updates an existing agent session.
    ///
    /// # Arguments
    /// * `session` - The updated session data
    ///
    /// # Errors
    /// Returns an error if the session cannot be updated (e.g., not found, storage failure).
    async fn update_session(&self, session: &AgentSession) -> Result<()>;

    /// Lists agent sessions matching the provided query filters.
    ///
    /// # Arguments
    /// * `query` - Query filters to apply
    ///
    /// # Returns
    /// A vector of sessions matching the query criteria.
    ///
    /// # Errors
    /// Returns an error if the query fails (e.g., storage error).
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;

    /// Stores a delegation record for an agent session.
    ///
    /// # Arguments
    /// * `delegation` - The delegation to store
    ///
    /// # Errors
    /// Returns an error if the delegation cannot be stored (e.g., storage failure).
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()>;

    /// Stores a tool call record for an agent session.
    ///
    /// # Arguments
    /// * `tool_call` - The tool call to store
    ///
    /// # Errors
    /// Returns an error if the tool call cannot be stored (e.g., storage failure).
    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()>;

    /// Stores a checkpoint for session state persistence.
    ///
    /// # Arguments
    /// * `checkpoint` - The checkpoint to store
    ///
    /// # Errors
    /// Returns an error if the checkpoint cannot be stored (e.g., storage failure).
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;

    /// Retrieves a checkpoint by ID.
    ///
    /// # Arguments
    /// * `id` - The checkpoint ID to retrieve
    ///
    /// # Returns
    /// `Ok(Some(checkpoint))` if found, `Ok(None)` if not found.
    ///
    /// # Errors
    /// Returns an error if the retrieval fails (e.g., storage error).
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;

    /// Updates an existing checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint` - The updated checkpoint data
    ///
    /// # Errors
    /// Returns an error if the checkpoint cannot be updated (e.g., not found, storage failure).
    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
}
