//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Provides workflow persistence repository domain definitions.
use async_trait::async_trait;

use crate::entities::{Transition, WorkflowSession, WorkflowState};
use crate::error::Result;

/// Port for workflow session persistence.
#[async_trait]
pub trait WorkflowSessionRepository: Send + Sync {
    /// Persists a new workflow session.
    async fn create(&self, session: &WorkflowSession) -> Result<()>;
    /// Fetches a workflow session by id.
    async fn get_by_id(&self, session_id: &str) -> Result<WorkflowSession>;
    /// Lists workflow sessions for a project.
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<WorkflowSession>>;
    /// Updates workflow state with optimistic concurrency using expected `version`.
    async fn update_state(
        &self,
        session_id: &str,
        new_state: WorkflowState,
        version: u32,
    ) -> Result<()>;
}

/// Port for workflow transition audit persistence.
#[async_trait]
pub trait TransitionRepository: Send + Sync {
    /// Records a transition event.
    async fn record(&self, transition: &Transition) -> Result<()>;
    /// Lists transitions for a workflow session.
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Transition>>;
}
