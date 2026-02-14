//! Unit tests for service lifecycle management
//!
//! Tests service state tracking, lifecycle operations, and the LifecycleManaged trait.
//! Includes a test implementation to verify trait behavior.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::admin::{
    DependencyHealth, DependencyHealthCheck, LifecycleManaged, PortServiceState,
};
use rstest::*;

// =============================================================================
// PortServiceState Enum Tests
// =============================================================================

#[rstest]
#[case(PortServiceState::Starting, "Starting")]
#[case(PortServiceState::Running, "Running")]
#[case(PortServiceState::Stopping, "Stopping")]
#[case(PortServiceState::Stopped, "Stopped")]
fn test_service_state_properties(#[case] state: PortServiceState, #[case] expected_debug: &str) {
    // Debug representation
    assert!(format!("{:?}", state).contains(expected_debug));

    // Clone/Copy
    assert_eq!(state, state.clone());

    // Serialization
    let json = serde_json::to_string(&state).expect("serialization failed");
    assert!(json.contains(expected_debug));

    // Deserialization
    let deserialized: PortServiceState =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized, state);
}

#[rstest]
fn test_all_service_states_distinct() {
    let states = [
        PortServiceState::Starting,
        PortServiceState::Running,
        PortServiceState::Stopping,
        PortServiceState::Stopped,
    ];

    for (i, state_a) in states.iter().enumerate() {
        for (j, state_b) in states.iter().enumerate() {
            if i != j {
                assert_ne!(state_a, state_b, "States at {} and {} should differ", i, j);
            }
        }
    }
}

#[test]
fn test_service_state_default() {
    assert_eq!(PortServiceState::default(), PortServiceState::Stopped);
}

// =============================================================================
// DependencyHealth Tests
// =============================================================================

#[rstest]
#[case(DependencyHealth::Healthy)]
#[case(DependencyHealth::Degraded)]
#[case(DependencyHealth::Unhealthy)]
#[case(DependencyHealth::Unknown)]
fn test_dependency_health_properties(#[case] health: DependencyHealth) {
    // Serialization round-trip
    let json = serde_json::to_string(&health).expect("serialization failed");
    let deserialized: DependencyHealth =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized, health);
}

#[test]
fn test_dependency_health_values_distinct() {
    assert_ne!(DependencyHealth::Healthy, DependencyHealth::Degraded);
    assert_ne!(DependencyHealth::Degraded, DependencyHealth::Unhealthy);
    assert_ne!(DependencyHealth::Unhealthy, DependencyHealth::Unknown);
}

#[test]
fn test_dependency_health_default() {
    assert_eq!(DependencyHealth::default(), DependencyHealth::Unknown);
}

// =============================================================================
// DependencyHealthCheck Tests
// =============================================================================

#[test]
fn test_health_check_default() {
    let check: DependencyHealthCheck = Default::default();
    assert_eq!(check.name, "");
    assert_eq!(check.status, DependencyHealth::Unknown);
    assert!(check.message.is_none());
    assert!(check.latency_ms.is_none());
    assert_eq!(check.last_check, 0);
}

#[rstest]
#[case("database", DependencyHealth::Healthy, Some("Connected".into()), Some(15))]
#[case("redis", DependencyHealth::Degraded, Some("High latency".into()), Some(500))]
fn test_health_check_properties(
    #[case] name: &str,
    #[case] status: DependencyHealth,
    #[case] message: Option<String>,
    #[case] latency_ms: Option<u64>,
) {
    let check = DependencyHealthCheck {
        name: name.to_string(),
        status,
        message: message.clone(),
        latency_ms,
        last_check: 1234567890,
    };

    assert_eq!(check.name, name);
    assert_eq!(check.status, status);
    assert_eq!(check.message, message);
    assert_eq!(check.latency_ms, latency_ms);

    // Serialization
    let json = serde_json::to_string(&check).expect("serialization failed");
    assert!(json.contains(name));

    let deserialized: DependencyHealthCheck =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.name, name);
    assert_eq!(deserialized.status, status);
}

// =============================================================================
// Test Implementation of LifecycleManaged
// =============================================================================

struct TestService {
    name: String,
    state: AtomicU32,
    start_count: AtomicU32,
    stop_count: AtomicU32,
    healthy: bool,
}

impl TestService {
    fn new(name: &str, healthy: bool) -> Self {
        Self {
            name: name.to_string(),
            state: AtomicU32::new(Self::state_to_u32(PortServiceState::Stopped)),
            start_count: AtomicU32::new(0),
            stop_count: AtomicU32::new(0),
            healthy,
        }
    }

    fn state_to_u32(state: PortServiceState) -> u32 {
        match state {
            PortServiceState::Starting => 0,
            PortServiceState::Running => 1,
            PortServiceState::Stopping => 2,
            PortServiceState::Stopped => 3,
        }
    }

    fn u32_to_state(value: u32) -> PortServiceState {
        match value {
            0 => PortServiceState::Starting,
            1 => PortServiceState::Running,
            2 => PortServiceState::Stopping,
            3 => PortServiceState::Stopped,
            _ => PortServiceState::Stopped,
        }
    }
}

#[async_trait]
impl LifecycleManaged for TestService {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> PortServiceState {
        Self::u32_to_state(self.state.load(Ordering::SeqCst))
    }

    async fn start(&self) -> Result<()> {
        self.state.store(
            Self::state_to_u32(PortServiceState::Starting),
            Ordering::SeqCst,
        );
        self.start_count.fetch_add(1, Ordering::SeqCst);
        // Simulate startup
        self.state.store(
            Self::state_to_u32(PortServiceState::Running),
            Ordering::SeqCst,
        );
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.state.store(
            Self::state_to_u32(PortServiceState::Stopping),
            Ordering::SeqCst,
        );
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        // Simulate shutdown
        self.state.store(
            Self::state_to_u32(PortServiceState::Stopped),
            Ordering::SeqCst,
        );
        Ok(())
    }

    async fn health_check(&self) -> DependencyHealthCheck {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.healthy {
            DependencyHealthCheck {
                name: self.name.clone(),
                status: DependencyHealth::Healthy,
                message: Some("Service is healthy".to_string()),
                latency_ms: Some(5),
                last_check: now,
            }
        } else {
            DependencyHealthCheck {
                name: self.name.clone(),
                status: DependencyHealth::Unhealthy,
                message: Some("Service check failed".to_string()),
                latency_ms: None,
                last_check: now,
            }
        }
    }
}

// =============================================================================
// LifecycleManaged Trait Tests
// =============================================================================

#[fixture]
fn service() -> TestService {
    TestService::new("test-service", true)
}

#[rstest]
#[tokio::test]
async fn test_service_lifecycle(service: TestService) {
    // Initial state
    assert_eq!(service.name(), "test-service");
    assert_eq!(service.state(), PortServiceState::Stopped);

    // Start
    service.start().await.expect("start should succeed");
    assert_eq!(service.state(), PortServiceState::Running);

    // Restart
    let initial_starts = service.start_count.load(Ordering::SeqCst);
    let initial_stops = service.stop_count.load(Ordering::SeqCst);
    service.restart().await.expect("restart should succeed");

    assert_eq!(
        service.start_count.load(Ordering::SeqCst),
        initial_starts + 1
    );
    assert_eq!(service.stop_count.load(Ordering::SeqCst), initial_stops + 1);
    assert_eq!(service.state(), PortServiceState::Running);

    // Stop
    service.stop().await.expect("stop should succeed");
    assert_eq!(service.state(), PortServiceState::Stopped);
}

#[rstest]
#[case(true, DependencyHealth::Healthy)]
#[case(false, DependencyHealth::Unhealthy)]
#[tokio::test]
async fn test_health_check_variations(
    #[case] healthy: bool,
    #[case] expected_status: DependencyHealth,
) {
    let service = TestService::new("health-test", healthy);
    let check = service.health_check().await;

    assert_eq!(check.name, "health-test");
    assert_eq!(check.status, expected_status);
    assert!(check.message.is_some());

    if healthy {
        assert!(check.latency_ms.is_some());
    } else {
        assert!(check.latency_ms.is_none());
    }
}

#[tokio::test]
async fn test_service_as_trait_object() {
    let service: Arc<dyn LifecycleManaged> = Arc::new(TestService::new("dynamic-service", true));

    assert_eq!(service.name(), "dynamic-service");
    assert_eq!(service.state(), PortServiceState::Stopped);

    service.start().await.expect("start should succeed");
    assert_eq!(service.state(), PortServiceState::Running);

    let health = service.health_check().await;
    assert_eq!(health.status, DependencyHealth::Healthy);

    service.stop().await.expect("stop should succeed");
    assert_eq!(service.state(), PortServiceState::Stopped);
}

#[tokio::test]
async fn test_multiple_services_independence() {
    let service_a = TestService::new("service-a", true);
    let service_b = TestService::new("service-b", false);

    service_a.start().await.expect("start A should succeed");

    assert_eq!(service_a.state(), PortServiceState::Running);
    assert_eq!(service_b.state(), PortServiceState::Stopped);

    let health_a = service_a.health_check().await;
    let health_b = service_b.health_check().await;

    assert_eq!(health_a.status, DependencyHealth::Healthy);
    assert_eq!(health_b.status, DependencyHealth::Unhealthy);
}
