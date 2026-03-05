//! Workflow session and transition repository ports.

use async_trait::async_trait;

use crate::entities::{Transition, WorkflowSession, WorkflowState};
use crate::error::Result;

/// Port for workflow session persistence.
#[async_trait]
pub trait WorkflowSessionRepository: Send + Sync {
    /// Persist a new workflow session.
    async fn create(&self, session: &WorkflowSession) -> Result<()>;
    /// Fetch a workflow session by ID.
    async fn get_by_id(&self, session_id: &str) -> Result<WorkflowSession>;
    /// List workflow sessions for a project.
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<WorkflowSession>>;
    /// Update workflow state with optimistic concurrency.
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
    /// Record a transition event.
    async fn record(&self, transition: &Transition) -> Result<()>;
    /// List transitions for a workflow session.
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Transition>>;
}
