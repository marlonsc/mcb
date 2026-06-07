use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use futures::stream;
use mcb_domain::entities::{TransitionTrigger, WorkflowSession, WorkflowState};
use mcb_domain::error::{Error, Result};
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{
    DomainEventStream, EventBusProvider, TransitionRepository, WorkflowSessionRepository,
};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_providers::workflow::{
    InMemoryTransitionRepository, InMemoryWorkflowSessionRepository, WorkflowEventPublisher,
    WorkflowOrchestrator,
};
use rstest::rstest;
use tokio::sync::Mutex;

struct TestEventBus {
    events: Arc<Mutex<Vec<DomainEvent>>>,
}

impl TestEventBus {
    fn new(events: Arc<Mutex<Vec<DomainEvent>>>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl EventBusProvider for TestEventBus {
    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        self.events.lock().await.push(event);
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        Ok(Box::pin(stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        true
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("sub-1".to_owned())
    }
}

struct ConflictOnUpdateSessionRepository {
    inner: Arc<InMemoryWorkflowSessionRepository>,
    fail_once: AtomicBool,
}

impl ConflictOnUpdateSessionRepository {
    fn new() -> Self {
        Self {
            inner: Arc::new(InMemoryWorkflowSessionRepository::default()),
            fail_once: AtomicBool::new(true),
        }
    }
}

#[async_trait]
impl WorkflowSessionRepository for ConflictOnUpdateSessionRepository {
    async fn create(&self, session: &WorkflowSession) -> Result<()> {
        self.inner.create(session).await
    }

    async fn get_by_id(&self, session_id: &str) -> Result<WorkflowSession> {
        self.inner.get_by_id(session_id).await
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<WorkflowSession>> {
        self.inner.list_by_project(project_id).await
    }

    async fn update_state(
        &self,
        session_id: &str,
        new_state: WorkflowState,
        version: u32,
    ) -> Result<()> {
        if self.fail_once.swap(false, Ordering::SeqCst) {
            return Err(Error::database(format!(
                "optimistic concurrency conflict for session {session_id} at version {version}"
            )));
        }

        self.inner
            .update_state(session_id, new_state, version)
            .await
    }
}

#[rstest]
#[tokio::test]
async fn full_workflow_lifecycle_persists_history_and_events() -> TestResult {
    let session_repo: Arc<dyn WorkflowSessionRepository> =
        Arc::new(InMemoryWorkflowSessionRepository::default());
    let transition_repo: Arc<dyn TransitionRepository> =
        Arc::new(InMemoryTransitionRepository::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TestEventBus::new(Arc::clone(&events)));
    let orchestrator = WorkflowOrchestrator::new(
        Arc::clone(&session_repo),
        Arc::clone(&transition_repo),
        WorkflowEventPublisher::new(event_bus),
    );

    let session = orchestrator.create_session("project-alpha").await?;

    let triggers = vec![
        TransitionTrigger::ContextDiscovered {
            context_id: "ctx-1".to_owned(),
        },
        TransitionTrigger::StartPlanning {
            phase_id: "phase-1".to_owned(),
        },
        TransitionTrigger::StartExecution {
            phase_id: "phase-1".to_owned(),
        },
        TransitionTrigger::StartVerification,
        TransitionTrigger::VerificationPassed,
        TransitionTrigger::EndSession,
    ];

    let mut final_state = WorkflowState::Initializing;
    for trigger in triggers {
        final_state = orchestrator.apply_trigger(&session.id, trigger).await?;
    }

    assert!(matches!(final_state, WorkflowState::Completed));

    let persisted = orchestrator.get_session(&session.id).await?;
    assert!(matches!(persisted.current_state, WorkflowState::Completed));
    assert_eq!(persisted.version, 6);

    let history = orchestrator.get_history(&session.id).await?;
    assert_eq!(history.len(), 6);

    let captured_events = events.lock().await;
    assert_eq!(captured_events.len(), 7);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn invalid_transition_returns_error_without_side_effects() -> TestResult {
    let session_repo: Arc<dyn WorkflowSessionRepository> =
        Arc::new(InMemoryWorkflowSessionRepository::default());
    let transition_repo: Arc<dyn TransitionRepository> =
        Arc::new(InMemoryTransitionRepository::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TestEventBus::new(Arc::clone(&events)));
    let orchestrator = WorkflowOrchestrator::new(
        Arc::clone(&session_repo),
        Arc::clone(&transition_repo),
        WorkflowEventPublisher::new(event_bus),
    );

    let session = orchestrator.create_session("project-alpha").await?;

    let result = orchestrator
        .apply_trigger(
            &session.id,
            TransitionTrigger::StartPlanning {
                phase_id: "phase-1".to_owned(),
            },
        )
        .await;

    let err = result.expect_err("invalid transition should fail");
    assert!(
        err.to_string().contains("Invalid FSM transition"),
        "unexpected error: {err}"
    );

    let persisted = orchestrator.get_session(&session.id).await?;
    assert!(matches!(
        persisted.current_state,
        WorkflowState::Initializing
    ));
    assert_eq!(persisted.version, 0);

    let history = orchestrator.get_history(&session.id).await?;
    assert_eq!(history.len(), 0);

    let captured_events = events.lock().await;
    assert_eq!(captured_events.len(), 1);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn optimistic_concurrency_conflict_returns_error_without_transition_side_effects()
-> TestResult {
    let session_repo: Arc<dyn WorkflowSessionRepository> =
        Arc::new(ConflictOnUpdateSessionRepository::new());
    let transition_repo: Arc<dyn TransitionRepository> =
        Arc::new(InMemoryTransitionRepository::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TestEventBus::new(Arc::clone(&events)));
    let orchestrator = WorkflowOrchestrator::new(
        Arc::clone(&session_repo),
        Arc::clone(&transition_repo),
        WorkflowEventPublisher::new(event_bus),
    );

    let session = orchestrator.create_session("project-alpha").await?;

    let result = orchestrator
        .apply_trigger(
            &session.id,
            TransitionTrigger::ContextDiscovered {
                context_id: "ctx-1".to_owned(),
            },
        )
        .await;

    let err = result.expect_err("concurrency conflict should fail");
    assert!(
        err.to_string().contains("optimistic concurrency"),
        "unexpected error: {err}"
    );

    let persisted = orchestrator.get_session(&session.id).await?;
    assert!(matches!(
        persisted.current_state,
        WorkflowState::Initializing
    ));
    assert_eq!(persisted.version, 0);

    let history = orchestrator.get_history(&session.id).await?;
    assert_eq!(history.len(), 0);

    let captured_events = events.lock().await;
    assert_eq!(captured_events.len(), 1);
    Ok(())
}
