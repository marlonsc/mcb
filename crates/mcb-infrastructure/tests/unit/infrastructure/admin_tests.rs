use mcb_domain::ports::IndexingOperationsInterface;
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::admin::DefaultIndexingOperations;
use rstest::rstest;
use rstest::*;

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
