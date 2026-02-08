use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::admin::{
    AtomicPerformanceMetrics, DefaultIndexingOperations,
};

#[test]
fn test_performance_metrics_initial_state() {
    let metrics = AtomicPerformanceMetrics::new();
    let data = metrics.get_performance_metrics();

    assert_eq!(data.total_queries, 0);
    assert_eq!(data.successful_queries, 0);
    assert_eq!(data.failed_queries, 0);
    assert_eq!(data.average_response_time_ms, 0.0);
    assert_eq!(data.cache_hit_rate, 0.0);
    assert_eq!(data.active_connections, 0);
}

#[test]
fn test_record_successful_query() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.record_query(100, true, false);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 1);
    assert_eq!(data.successful_queries, 1);
    assert_eq!(data.failed_queries, 0);
    assert_eq!(data.average_response_time_ms, 100.0);
    assert_eq!(data.cache_hit_rate, 0.0);
}

#[test]
fn test_record_failed_query() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.record_query(50, false, false);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 1);
    assert_eq!(data.successful_queries, 0);
    assert_eq!(data.failed_queries, 1);
}

#[test]
fn test_record_query_with_cache_hit() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.record_query(10, true, true);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 1);
    assert_eq!(data.cache_hit_rate, 1.0);
}

#[test]
fn test_multiple_queries_average_response_time() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.record_query(100, true, false);
    metrics.record_query(200, true, false);
    metrics.record_query(300, true, true);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.total_queries, 3);
    assert_eq!(data.successful_queries, 3);
    assert_eq!(data.average_response_time_ms, 200.0);
    assert!((data.cache_hit_rate - 1.0 / 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_update_active_connections_increment() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.update_active_connections(3);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.active_connections, 3);
}

#[test]
fn test_update_active_connections_decrement() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.update_active_connections(5);
    metrics.update_active_connections(-2);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.active_connections, 3);
}

#[test]
fn test_update_active_connections_no_underflow() {
    let metrics = AtomicPerformanceMetrics::new();
    metrics.update_active_connections(2);
    metrics.update_active_connections(-10);

    let data = metrics.get_performance_metrics();
    assert_eq!(data.active_connections, 0);
}

#[test]
fn test_uptime_nonzero() {
    let metrics = AtomicPerformanceMetrics::new();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(metrics.uptime_secs() < 5);
}

#[test]
fn test_default_matches_new() {
    let a = AtomicPerformanceMetrics::new();
    let b = AtomicPerformanceMetrics::default();
    let da = a.get_performance_metrics();
    let db = b.get_performance_metrics();
    assert_eq!(da.total_queries, db.total_queries);
    assert_eq!(da.active_connections, db.active_connections);
}

#[test]
fn test_indexing_operations_start_and_get() {
    let tracker = DefaultIndexingOperations::new();
    let collection = CollectionId::new("test-collection".to_string());
    let op_id = tracker.start_operation(&collection, 10);

    let ops = tracker.get_operations();
    assert_eq!(ops.len(), 1);
    let op = &ops[&op_id];
    assert_eq!(op.total_files, 10);
    assert_eq!(op.processed_files, 0);
}

#[test]
fn test_indexing_operations_update_progress() {
    let tracker = DefaultIndexingOperations::new();
    let collection = CollectionId::new("coll".to_string());
    let op_id = tracker.start_operation(&collection, 5);

    tracker.update_progress(&op_id, Some("file1.rs".to_string()), 2);

    let ops = tracker.get_operations();
    let op = &ops[&op_id];
    assert_eq!(op.processed_files, 2);
    assert_eq!(op.current_file.as_deref(), Some("file1.rs"));
}

#[test]
fn test_indexing_operations_complete_removes() {
    let tracker = DefaultIndexingOperations::new();
    let collection = CollectionId::new("coll".to_string());
    let op_id = tracker.start_operation(&collection, 3);
    assert!(tracker.has_active_operations());
    assert_eq!(tracker.active_count(), 1);

    tracker.complete_operation(&op_id);
    assert!(!tracker.has_active_operations());
    assert_eq!(tracker.active_count(), 0);
    assert!(tracker.get_operations().is_empty());
}

#[test]
fn test_indexing_operations_multiple() {
    let tracker = DefaultIndexingOperations::new();
    let c1 = CollectionId::new("c1".to_string());
    let c2 = CollectionId::new("c2".to_string());

    let _op1 = tracker.start_operation(&c1, 10);
    let _op2 = tracker.start_operation(&c2, 20);

    assert_eq!(tracker.active_count(), 2);
    assert_eq!(tracker.get_operations().len(), 2);
}

#[test]
fn test_indexing_operations_default() {
    let tracker = DefaultIndexingOperations::default();
    assert!(!tracker.has_active_operations());
    assert_eq!(tracker.active_count(), 0);
}
