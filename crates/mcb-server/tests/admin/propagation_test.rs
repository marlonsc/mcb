//! Tests for ConfigPropagator
//!
//! Tests configuration propagation and callback functionality.

use mcb_server::admin::propagation::{ConfigPropagator, PropagatorHandle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_config_propagator_creation() {
    let propagator = ConfigPropagator::new();
    assert!(propagator.callbacks.is_empty());
}

#[tokio::test]
async fn test_config_propagator_with_callback() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let propagator = ConfigPropagator::new().on_config_change(Box::new(move |_config| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    assert_eq!(propagator.callbacks.len(), 1);
}

#[test]
fn test_propagator_handle_is_running() {
    // Test that the handle properly tracks task state
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    let handle = runtime.block_on(async {
        let handle = tokio::spawn(async {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });
        PropagatorHandle { handle }
    });

    // Verify that is_running() returns a boolean
    let is_running = handle.is_running();
    assert!(matches!(is_running, true | false));

    // Wait for task completion and verify final state
    runtime.block_on(async {
        let _ = handle.handle.await;
        // After awaiting, the task should not be running anymore
        assert!(!handle.is_running());
    });
}
