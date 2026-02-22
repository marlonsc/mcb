//! Unit tests for provider routing and health monitoring
//!
//! Tests the `InMemoryHealthMonitor` state transitions and `DefaultProviderRouter`
//! provider selection logic based on health status.

use rstest::rstest;
use std::sync::Arc;

use mcb_domain::ports::{ProviderContext, ProviderHealthStatus, ProviderRouter};
use mcb_infrastructure::routing::{DefaultProviderRouter, HealthMonitor, InMemoryHealthMonitor};
use rstest::*;

// =============================================================================
// InMemoryHealthMonitor Tests
// =============================================================================

#[fixture]
fn monitor() -> InMemoryHealthMonitor {
    InMemoryHealthMonitor::new()
}

#[rstest]
fn test_new_provider_starts_healthy(monitor: InMemoryHealthMonitor) {
    let status = monitor.get_health("unknown-provider");
    assert_eq!(status, ProviderHealthStatus::Healthy);
}

#[rstest]
#[case(2, ProviderHealthStatus::Degraded)]
#[case(5, ProviderHealthStatus::Unhealthy)]
fn test_health_transitions_failure_count(
    monitor: InMemoryHealthMonitor,
    #[case] failure_count: usize,
    #[case] expected: ProviderHealthStatus,
) {
    for _ in 0..failure_count {
        monitor.record_failure("provider-a");
    }
    assert_eq!(monitor.get_health("provider-a"), expected);
}

#[rstest]
fn test_success_resets_to_healthy(monitor: InMemoryHealthMonitor) {
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a"); // Degraded
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Degraded
    );

    monitor.record_success("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Healthy
    );
}

#[rstest]
fn test_custom_thresholds() {
    let monitor = InMemoryHealthMonitor::with_thresholds(1, 3);

    monitor.record_failure("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Degraded
    );

    monitor.record_failure("provider-a"); // 2
    monitor.record_failure("provider-a"); // 3
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Unhealthy
    );
}

#[rstest]
fn test_get_all_health(monitor: InMemoryHealthMonitor) {
    monitor.record_success("provider-a");
    monitor.record_failure("provider-b");
    monitor.record_failure("provider-b"); // Degraded

    let all_health = monitor.get_all_health();
    assert_eq!(all_health.len(), 2);
    assert_eq!(
        all_health.get("provider-a"),
        Some(&ProviderHealthStatus::Healthy)
    );
    assert_eq!(
        all_health.get("provider-b"),
        Some(&ProviderHealthStatus::Degraded)
    );
}

// =============================================================================
// DefaultProviderRouter Tests
// =============================================================================

#[fixture]
fn router_setup() -> (Arc<InMemoryHealthMonitor>, DefaultProviderRouter) {
    let monitor = Arc::new(InMemoryHealthMonitor::new());
    let router = DefaultProviderRouter::new(
        Arc::clone(&monitor),
        vec!["provider-a".to_owned(), "provider-b".to_owned()],
        vec![],
    );
    (monitor, router)
}

#[rstest]
#[tokio::test]
async fn test_router_selection_scenarios(
    router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter),
) {
    let (monitor, router) = router_setup;

    // Case 1: Prefers healthy over unhealthy
    for _ in 0..5 {
        monitor.record_failure("provider-a");
    } // A Unhealthy
    monitor.record_success("provider-b"); // B Healthy

    let ctx = ProviderContext::new();
    let selected = router.select_embedding_provider(&ctx).await.unwrap();
    assert_eq!(selected, "provider-b");

    // Case 2: Prefers degraded over unhealthy
    monitor.record_success("provider-a"); // Reset A
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a"); // A Degraded
    for _ in 0..5 {
        monitor.record_failure("provider-b");
    } // B Unhealthy

    let selected = router.select_embedding_provider(&ctx).await.unwrap();
    assert_eq!(selected, "provider-a");
}

#[rstest]
#[tokio::test]
async fn test_router_excludes_providers(
    router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter),
) {
    let (_, router) = router_setup;
    let ctx = ProviderContext::new().exclude("provider-a");
    let selected = router.select_embedding_provider(&ctx).await.unwrap();
    assert_eq!(selected, "provider-b");
}

#[rstest]
#[tokio::test]
async fn test_router_prefers_preferred_provider(
    router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter),
) {
    let (_, router) = router_setup;
    // Both healthy
    let ctx = ProviderContext::new().prefer("provider-b");
    assert_eq!(
        router.select_embedding_provider(&ctx).await.unwrap(),
        "provider-b"
    );
}

#[rstest]
#[tokio::test]
async fn test_router_fallback_when_preferred_unhealthy(
    router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter),
) {
    let (monitor, router) = router_setup;
    for _ in 0..5 {
        monitor.record_failure("provider-b");
    } // B Unhealthy

    let ctx = ProviderContext::new().prefer("provider-b");
    // Should fallback to A
    assert_eq!(
        router.select_embedding_provider(&ctx).await.unwrap(),
        "provider-a"
    );
}

#[rstest]
#[tokio::test]
async fn test_router_reporting(router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter)) {
    let (_, router) = router_setup;

    for _ in 0..5 {
        router
            .report_failure("provider-a", "timeout")
            .await
            .unwrap();
    }
    assert_eq!(
        router.get_provider_health("provider-a").await.unwrap(),
        ProviderHealthStatus::Unhealthy
    );

    router.report_success("provider-a").await.unwrap();
    assert_eq!(
        router.get_provider_health("provider-a").await.unwrap(),
        ProviderHealthStatus::Healthy
    );
}

#[rstest]
#[tokio::test]
async fn test_router_error_no_providers(
    router_setup: (Arc<InMemoryHealthMonitor>, DefaultProviderRouter),
) {
    let (_, router) = router_setup;
    // Exclude all
    let ctx = ProviderContext::new()
        .exclude("provider-a")
        .exclude("provider-b");
    assert!(router.select_embedding_provider(&ctx).await.is_err());
}
