//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Workflow orchestrator, event publisher, and in-memory repository implementations.
//!
//! The [`WorkflowEventPublisher`] bridges the workflow module to the domain event
//! system by publishing [`DomainEvent`] variants through an [`EventBusProvider`].
//! It gracefully degrades: when no subscribers are listening, serialization is skipped.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mcb_domain::entities::workflow::TransitionInput;
use mcb_domain::entities::{Transition, TransitionTrigger, WorkflowSession, WorkflowState};
use mcb_domain::error::{Error, Result};
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{EventBusProvider, TransitionRepository, WorkflowSessionRepository};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use crate::workflow::transitions::apply_transition;

// ---------------------------------------------------------------------------
// WorkflowEvent — lightweight local enum kept for backward compatibility
// ---------------------------------------------------------------------------

/// Internal workflow event type (kept for serialization compatibility).
///
/// New code should prefer [`WorkflowEventPublisher`] which publishes
/// [`DomainEvent`] variants directly to the event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// A new workflow session was created.
    SessionCreated {
        /// Unique session identifier.
        session_id: String,
        /// Project the session belongs to.
        project_id: String,
    },
    /// A state transition was applied to a session.
    TransitionApplied {
        /// Unique session identifier.
        session_id: String,
        /// State before the transition.
        from_state: WorkflowState,
        /// State after the transition.
        to_state: WorkflowState,
        /// Trigger that caused the transition.
        trigger: TransitionTrigger,
    },
}

// ---------------------------------------------------------------------------
// WorkflowEventPublisher — bridges workflow → DomainEvent system
// ---------------------------------------------------------------------------

/// Publishes workflow lifecycle events through the domain event bus.
///
/// Gracefully degrades: if [`EventBusProvider::has_subscribers`] returns `false`,
/// event construction and serialization are skipped entirely.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use mcb_providers::workflow::WorkflowEventPublisher;
/// # fn example(event_bus: Arc<dyn mcb_domain::ports::EventBusProvider>) {
/// let publisher = WorkflowEventPublisher::new(event_bus);
/// // publisher.publish_session_created("sess-1", "proj-1").await?;
/// # }
/// ```
#[derive(Clone)]
pub struct WorkflowEventPublisher {
    event_bus: Arc<dyn EventBusProvider>,
}

impl WorkflowEventPublisher {
    /// Create a new publisher backed by the given event bus.
    #[must_use]
    pub fn new(event_bus: Arc<dyn EventBusProvider>) -> Self {
        Self { event_bus }
    }

    /// Publish a session-created event.
    ///
    /// Skips publishing if no subscribers are listening.
    ///
    /// # Errors
    ///
    /// Returns an error if event publishing fails.
    pub async fn publish_session_created(&self, session_id: &str, project_id: &str) -> Result<()> {
        if !self.event_bus.has_subscribers() {
            debug!("Skipping WorkflowSessionCreated event — no subscribers");
            return Ok(());
        }

        self.event_bus
            .publish_event(DomainEvent::WorkflowSessionCreated {
                session_id: session_id.to_owned(),
                project_id: project_id.to_owned(),
            })
            .await
    }

    /// Publish a state transition event.
    ///
    /// Uses `Display` formatting for state and trigger values.
    /// Skips publishing if no subscribers are listening.
    ///
    /// # Errors
    ///
    /// Returns an error if event publishing fails.
    pub async fn publish_transition(
        &self,
        session_id: &str,
        from_state: &WorkflowState,
        to_state: &WorkflowState,
        trigger: &TransitionTrigger,
    ) -> Result<()> {
        if !self.event_bus.has_subscribers() {
            debug!("Skipping WorkflowTransitioned event — no subscribers");
            return Ok(());
        }

        self.event_bus
            .publish_event(DomainEvent::WorkflowTransitioned {
                session_id: session_id.to_owned(),
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
                trigger: trigger.to_string(),
            })
            .await
    }

    /// Publish a session-completed event.
    ///
    /// Skips publishing if no subscribers are listening.
    ///
    /// # Errors
    ///
    /// Returns an error if event publishing fails.
    pub async fn publish_session_completed(
        &self,
        session_id: &str,
        final_state: &WorkflowState,
        duration_ms: u64,
    ) -> Result<()> {
        if !self.event_bus.has_subscribers() {
            debug!("Skipping WorkflowSessionCompleted event — no subscribers");
            return Ok(());
        }

        self.event_bus
            .publish_event(DomainEvent::WorkflowSessionCompleted {
                session_id: session_id.to_owned(),
                final_state: final_state.to_string(),
                duration_ms,
            })
            .await
    }

    /// Publish a session-failed event.
    ///
    /// Skips publishing if no subscribers are listening.
    ///
    /// # Errors
    ///
    /// Returns an error if event publishing fails.
    pub async fn publish_session_failed(
        &self,
        session_id: &str,
        error: &str,
        recoverable: bool,
    ) -> Result<()> {
        if !self.event_bus.has_subscribers() {
            debug!("Skipping WorkflowSessionFailed event — no subscribers");
            return Ok(());
        }

        self.event_bus
            .publish_event(DomainEvent::WorkflowSessionFailed {
                session_id: session_id.to_owned(),
                error: error.to_owned(),
                recoverable,
            })
            .await
    }
}

impl std::fmt::Debug for WorkflowEventPublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEventPublisher")
            .field("has_subscribers", &self.event_bus.has_subscribers())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// WorkflowOrchestrator
// ---------------------------------------------------------------------------

/// Orchestrates workflow sessions: creation, transitions, and event publishing.
pub struct WorkflowOrchestrator {
    session_repo: Arc<dyn WorkflowSessionRepository>,
    transition_repo: Arc<dyn TransitionRepository>,
    event_publisher: WorkflowEventPublisher,
}

impl WorkflowOrchestrator {
    #[must_use]
    /// Create a new orchestrator with the given repositories and event publisher.
    pub fn new(
        session_repo: Arc<dyn WorkflowSessionRepository>,
        transition_repo: Arc<dyn TransitionRepository>,
        event_publisher: WorkflowEventPublisher,
    ) -> Self {
        Self {
            session_repo,
            transition_repo,
            event_publisher,
        }
    }

    /// Create a new workflow session for the given project.
    ///
    /// # Errors
    ///
    /// Returns an error if session creation or event publishing fails.
    pub async fn create_session(&self, project_id: &str) -> Result<WorkflowSession> {
        let session = WorkflowSession::new(Uuid::new_v4().to_string(), project_id.to_owned());

        self.session_repo.create(&session).await?;

        self.event_publisher
            .publish_session_created(&session.id, &session.project_id)
            .await?;

        Ok(session)
    }

    /// Apply a transition trigger to a session, persisting state and history.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found, the transition is invalid,
    /// or persistence/event publishing fails.
    pub async fn apply_trigger(
        &self,
        session_id: &str,
        trigger: TransitionTrigger,
    ) -> Result<WorkflowState> {
        let mut session = self.session_repo.get_by_id(session_id).await?;
        let from_state = session.current_state.clone();

        let new_state =
            apply_transition(&mut session, &trigger).map_err(Error::invalid_argument)?;

        self.session_repo
            .update_state(session_id, new_state.clone(), session.version)
            .await?;

        let transition = Transition::new(TransitionInput {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_owned(),
            from_state: from_state.clone(),
            to_state: new_state.clone(),
            trigger: trigger.clone(),
            guard_result: None,
        });

        self.transition_repo.record(&transition).await?;

        self.event_publisher
            .publish_transition(session_id, &from_state, &new_state, &trigger)
            .await?;

        Ok(new_state)
    }

    /// Retrieve a workflow session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub async fn get_session(&self, session_id: &str) -> Result<WorkflowSession> {
        self.session_repo.get_by_id(session_id).await
    }

    /// Retrieve the full transition history for a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the history cannot be retrieved.
    pub async fn get_history(&self, session_id: &str) -> Result<Vec<Transition>> {
        self.transition_repo.list_by_session(session_id).await
    }
}

// ---------------------------------------------------------------------------
// In-Memory Repositories
// ---------------------------------------------------------------------------

#[derive(Default)]
/// In-memory workflow session repository for testing.
pub struct InMemoryWorkflowSessionRepository {
    sessions: RwLock<HashMap<String, WorkflowSession>>,
}

#[async_trait]
impl WorkflowSessionRepository for InMemoryWorkflowSessionRepository {
    async fn create(&self, session: &WorkflowSession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session.id) {
            return Err(Error::invalid_argument(format!(
                "workflow session already exists: {}",
                session.id
            )));
        }

        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn get_by_id(&self, session_id: &str) -> Result<WorkflowSession> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("workflow session {session_id}")))
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<WorkflowSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|session| session.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn update_state(
        &self,
        session_id: &str,
        new_state: WorkflowState,
        version: u32,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("workflow session {session_id}")))?;

        if session.version != version {
            return Err(Error::database(format!(
                "optimistic concurrency conflict for session {session_id}: expected version {version}, current version {}",
                session.version
            )));
        }

        session.current_state = new_state;
        session.version = session.version.saturating_add(1);
        session.updated_at = Utc::now().timestamp();
        Ok(())
    }
}

#[derive(Default)]
/// In-memory transition repository for testing.
pub struct InMemoryTransitionRepository {
    transitions: RwLock<HashMap<String, Vec<Transition>>>,
}

#[async_trait]
impl TransitionRepository for InMemoryTransitionRepository {
    async fn record(&self, transition: &Transition) -> Result<()> {
        let mut transitions = self.transitions.write().await;
        transitions
            .entry(transition.session_id.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());
        Ok(())
    }

    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Transition>> {
        let transitions = self.transitions.read().await;
        Ok(transitions.get(session_id).cloned().unwrap_or_default())
    }
}
