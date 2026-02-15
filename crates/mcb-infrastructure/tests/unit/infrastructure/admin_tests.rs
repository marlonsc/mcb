use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::admin::{
    AtomicPerformanceMetrics, DefaultIndexingOperations,
};
use rstest::rstest;
use rstest::*;

#[fixture]
fn metrics() -> AtomicPerformanceMetrics {
    AtomicPerformanceMetrics::new()
}

#[rstest]
fn test_performance_metrics_initial_state(metrics: AtomicPerformanceMetrics) {
    let data = metrics.get_performance_metrics();

    assert_eq!(data.total_queries, 0);
    assert_eq!(data.successful_queries, 0);
    assert_eq!(data.failed_queries, 0);
    assert_eq!(data.average_response_time_ms, 0.0);
    assert_eq!(data.cache_hit_rate, 0.0);
    assert_eq!(data.active_connections, 0);
}

#[rstest]
#[case((100, true, false), (1, 1, 0, 100.0, 0.0))]
#[case((50, false, false), (1, 0, 1, 0.0, 0.0))]
fn test_record_query(
    metrics: AtomicPerformanceMetrics,
    #[case] input: (u64, bool, bool),
    #[case] expected: (u64, u64, u64, f64, f64),
) {
    let (time_ms, success, cache_hit) = input;
    let (
        expected_total,
        expected_success,
        expected_failed,
        expected_avg_time,
        expected_cache_hit_rate,
    ) = expected;

    metrics.record_query(time_ms, success, cache_hit);
    let data = metrics.get_performance_metrics();

    assert_eq!(data.total_queries, expected_total);
    assert_eq!(data.successful_queries, expected_success);
    assert_eq!(data.failed_queries, expected_failed);
    if success {
        assert!((data.average_response_time_ms - expected_avg_time).abs() < 0.001);
    }
    assert!((data.cache_hit_rate - expected_cache_hit_rate).abs() < 0.001);
}

#[rstest]
fn test_record_query_with_cache_hit(metrics: AtomicPerformanceMetrics) {
    metrics.record_query(10, true, true);
    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 1);
    assert_eq!(data.cache_hit_rate, 1.0);
}

#[rstest]
fn test_multiple_queries_aggregation(metrics: AtomicPerformanceMetrics) {
    metrics.record_query(100, true, false);
    metrics.record_query(200, true, false);
    metrics.record_query(300, true, true);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 3);
    assert_eq!(data.successful_queries, 3);
    assert_eq!(data.average_response_time_ms, 200.0);
    assert!((data.cache_hit_rate - 1.0 / 3.0).abs() < f64::EPSILON);
}

#[rstest]
#[case(3, 3)]
#[case(-2, 0)] // Default 0 + -2 -> 0 (no underflow)
fn test_update_active_connections_simple(
    metrics: AtomicPerformanceMetrics,
    #[case] delta: i64,
    #[case] expected: usize,
) {
    metrics.update_active_connections(delta);
    assert_eq!(
        metrics.get_performance_metrics().active_connections,
        expected
    );
}

#[rstest]
fn test_update_active_connections_sequence(metrics: AtomicPerformanceMetrics) {
    metrics.update_active_connections(5);
    metrics.update_active_connections(-2);
    assert_eq!(metrics.get_performance_metrics().active_connections, 3);
}

#[rstest]
fn test_update_active_connections_underflow_protection(metrics: AtomicPerformanceMetrics) {
    metrics.update_active_connections(2);
    metrics.update_active_connections(-10);
    assert_eq!(metrics.get_performance_metrics().active_connections, 0);
}

#[rstest]
fn test_uptime_nonzero(metrics: AtomicPerformanceMetrics) {
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(metrics.uptime_secs() < 5);
}

#[fixture]
fn tracker() -> DefaultIndexingOperations {
    DefaultIndexingOperations::new()
}

#[fixture]
fn collection() -> CollectionId {
    CollectionId::from_name("test-collection")
}

#[rstest]
fn test_indexing_operations_lifecycle(
    tracker: DefaultIndexingOperations,
    collection: CollectionId,
) {
    // Start
    let op_id = tracker.start_operation(&collection, 10);
    assert_eq!(tracker.active_count(), 1);

    let ops = tracker.get_operations();
    let op = &ops[&op_id];
    assert_eq!(op.total_files, 10);
    assert_eq!(op.processed_files, 0);

    // Update
    tracker.update_progress(&op_id, Some("file1.rs".to_owned()), 2);
    let ops = tracker.get_operations();
    let op = &ops[&op_id];
    assert_eq!(op.processed_files, 2);
    assert_eq!(op.current_file.as_deref(), Some("file1.rs"));

    // Complete
    tracker.complete_operation(&op_id);
    assert!(!tracker.has_active_operations());
    assert!(tracker.get_operations().is_empty());
}

#[rstest]
fn test_indexing_operations_multiple(tracker: DefaultIndexingOperations) {
    let c1 = CollectionId::from_name("c1");
    let c2 = CollectionId::from_name("c2");

    let _op1 = tracker.start_operation(&c1, 10);
    let _op2 = tracker.start_operation(&c2, 20);

    assert_eq!(tracker.active_count(), 2);
    assert_eq!(tracker.get_operations().len(), 2);
}
