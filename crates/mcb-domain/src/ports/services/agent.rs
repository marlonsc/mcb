//! Agent Service Port
//!
//! # Overview
//! Defines the interface for managing agent session lifecycle, delegation tracking,
//! and state checkpoints.
use async_trait::async_trait;

use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::error::Result;
use crate::ports::repositories::agent::AgentSessionQuery;

/// Port for agent session lifecycle and delegation tracking.
#[async_trait]
/// Manages agent session lifecycle.
pub trait AgentSessionManager: Send + Sync {
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
}

#[async_trait]
/// Tracks delegations and tool calls.
pub trait DelegationTracker: Send + Sync {
    /// Performs the store delegation operation.
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    /// Performs the store tool call operation.
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
}

#[async_trait]
/// Manages checkpoints and restoration.
pub trait CheckpointManager: Send + Sync {
    /// Performs the store checkpoint operation.
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    /// Performs the get checkpoint operation.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Performs the restore checkpoint operation.
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
