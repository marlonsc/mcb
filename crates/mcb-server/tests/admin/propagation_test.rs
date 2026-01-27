//! Tests for ConfigPropagator
//!
//! Tests configuration propagation and callback functionality.

use mcb_server::admin::propagation::{ConfigPropagator, PropagatorHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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

#[tokio::test]
async fn test_propagator_handle_is_running() {
    // Test that the handle properly tracks task state
    let handle = tokio::spawn(async {
        // Simulate work that takes time
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    });
    let propagator_handle = PropagatorHandle { handle };

    // Task should be running immediately after spawn
    assert!(
        propagator_handle.is_running(),
        "Task should be running immediately after spawn"
    );

    // Wait for task to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Task should be finished after sleep
    assert!(
        !propagator_handle.is_running(),
        "Task should be finished after completion"
    );
}
