use mcb_domain::ports::IndexingOperationsInterface;
use mcb_domain::registry::admin_operations::{
    IndexingOperationsProviderConfig, resolve_indexing_operations_provider,
};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::value_objects::CollectionId;
use rstest::{fixture, rstest};

#[fixture]
fn tracker() -> TestResult<std::sync::Arc<dyn IndexingOperationsInterface>> {
    resolve_indexing_operations_provider(&IndexingOperationsProviderConfig::new("default"))
        .map_err(Into::into)
}

#[fixture]
fn collection() -> CollectionId {
    CollectionId::from_name("test-collection")
}

#[rstest]
fn test_indexing_operations_lifecycle(
    tracker: TestResult<std::sync::Arc<dyn IndexingOperationsInterface>>,
    collection: CollectionId,
) -> TestResult {
    let tracker = tracker?;
    let op_id = tracker.start_operation(&collection, 10);

    let ops = tracker.get_operations();
    let op = &ops[&op_id];
    assert_eq!(op.total_files, 10);
    assert_eq!(op.processed_files, 0);

    tracker.update_progress(&op_id, Some("file1.rs".to_owned()), 2);
    let ops = tracker.get_operations();
    let op = &ops[&op_id];
    assert_eq!(op.processed_files, 2);
    assert_eq!(op.current_file.as_deref(), Some("file1.rs"));

    tracker.complete_operation(&op_id);
    assert!(tracker.get_operations().is_empty());
    Ok(())
}

#[rstest]
fn test_indexing_operations_multiple(
    tracker: TestResult<std::sync::Arc<dyn IndexingOperationsInterface>>,
) -> TestResult {
    let tracker = tracker?;
    let c1 = CollectionId::from_name("c1");
    let c2 = CollectionId::from_name("c2");

    let _op1 = tracker.start_operation(&c1, 10);
    let _op2 = tracker.start_operation(&c2, 20);

    assert_eq!(tracker.get_operations().len(), 2);
    Ok(())
}
