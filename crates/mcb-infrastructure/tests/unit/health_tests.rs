//! Health Check Tests

use mcb_infrastructure::health::{
    HealthCheck, HealthChecker, HealthRegistry, HealthResponse, HealthStatus, checkers,
};
use rstest::*;

#[rstest]
#[case(HealthCheck::healthy("test"), HealthStatus::Up, None)]
#[case(HealthCheck::failed("test", Some("err".into())), HealthStatus::Down, Some("err"))]
#[tokio::test]
async fn test_health_check_creation(
    #[case] check: HealthCheck,
    #[case] expected_status: HealthStatus,
    #[case] expected_err: Option<&str>,
) {
    assert_eq!(check.status, expected_status);
    assert_eq!(check.error.as_deref(), expected_err);
}

#[tokio::test]
async fn test_health_response_aggregation() {
    let response = HealthResponse::new()
        .add_check(HealthCheck::healthy("check1"))
        .add_check(HealthCheck::healthy("check2"));

    assert_eq!(response.status, HealthStatus::Up);
    assert_eq!(response.checks.len(), 2);

    let degraded_response = response.add_check(HealthCheck::degraded("check3", None));
    assert_eq!(degraded_response.status, HealthStatus::Degraded);
}

#[fixture]
async fn health_registry() -> HealthRegistry {
    HealthRegistry::new()
}

#[rstest]
#[tokio::test]
async fn test_health_registry_registration_and_execution(
    #[future] health_registry: HealthRegistry,
) {
    let registry = health_registry.await;
    registry
        .register_checker(
            "test".to_string(),
            checkers::ServiceHealthChecker::new("test", || Ok(())),
        )
        .await;

    let response = registry.perform_health_checks().await;
    assert_eq!(response.checks.len(), 1);
    assert!(response.checks["test"].status.is_healthy());

    let checks = registry.list_checks().await;
    assert_eq!(checks, vec!["test"]);
}

#[rstest]
#[case(HealthStatus::Up, true)]
#[case(HealthStatus::Degraded, true)]
#[case(HealthStatus::Down, false)]
fn test_health_status_predicates(#[case] status: HealthStatus, #[case] is_operational: bool) {
    assert_eq!(status.is_operational(), is_operational);
    if status == HealthStatus::Up {
        assert!(status.is_healthy());
    } else if status == HealthStatus::Down {
        assert!(!status.is_healthy());
    }
}

#[test]
fn test_status_specifics() {
    assert!(HealthStatus::Up.is_healthy());
    assert!(!HealthStatus::Down.is_healthy());
}

#[fixture]
fn system_checker() -> checkers::SystemHealthChecker {
    checkers::SystemHealthChecker::new()
}

#[rstest]
#[tokio::test]
async fn test_system_health_checker_structure(system_checker: checkers::SystemHealthChecker) {
    let result = system_checker.check_health().await;
    assert_eq!(result.name, "system");
    assert!(matches!(
        result.status,
        HealthStatus::Up | HealthStatus::Degraded | HealthStatus::Down
    ));
    assert!(result.details.is_some());
}

#[rstest]
#[tokio::test]
async fn test_real_cpu_metrics(system_checker: checkers::SystemHealthChecker) {
    let result = system_checker.check_health().await;
    let details = result.details.as_ref().unwrap();
    let cpu = details["cpu_usage_percent"].as_f64().unwrap();
    assert!((0.0..=100.0).contains(&cpu));
}

#[rstest]
#[tokio::test]
async fn test_real_memory_metrics(system_checker: checkers::SystemHealthChecker) {
    let result = system_checker.check_health().await;
    let details = result.details.as_ref().unwrap();

    let used = details["memory_used_bytes"].as_u64().unwrap();
    let total = details["memory_total_bytes"].as_u64().unwrap();
    let percent = details["memory_usage_percent"].as_f64().unwrap();

    assert!(total > 1_000_000_000); // > 1GB
    assert!(used > 0);
    assert!(used <= total);
    assert!((0.0..=100.0).contains(&percent));

    let expected = (used as f64 / total as f64) * 100.0;
    assert!((percent - expected).abs() < 0.01);
}

#[rstest]
#[case(80.0, 85.0)]
#[tokio::test]
async fn test_system_checker_thresholds(#[case] cpu_limit: f32, #[case] mem_limit: f64) {
    let checker = checkers::SystemHealthChecker::with_thresholds(cpu_limit, mem_limit);
    let result = checker.check_health().await;
    let details = result.details.as_ref().unwrap();

    assert_eq!(
        details["cpu_threshold_percent"].as_f64().unwrap(),
        cpu_limit as f64
    );
    assert_eq!(
        details["memory_threshold_percent"].as_f64().unwrap(),
        mem_limit
    );
}

#[rstest]
#[tokio::test]
async fn test_system_checker_response_time(system_checker: checkers::SystemHealthChecker) {
    let result = system_checker.check_health().await;
    assert!(result.response_time_ms >= 100);
}

#[rstest]
#[tokio::test]
async fn test_system_checker_required_fields(system_checker: checkers::SystemHealthChecker) {
    let result = system_checker.check_health().await;
    let details = result.details.as_ref().unwrap();

    let fields = [
        "cpu_usage_percent",
        "memory_used_bytes",
        "memory_total_bytes",
        "memory_usage_percent",
        "cpu_threshold_percent",
        "memory_threshold_percent",
    ];

    for field in fields {
        assert!(details.get(field).is_some(), "Missing field: {}", field);
    }
}
