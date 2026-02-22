//! Tests for `ConfigPropagator`
//!
//! Tests configuration propagation and callback functionality.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mcb_server::admin::propagation::{ConfigPropagator, PropagatorHandle};
use rstest::rstest;

#[rstest]
#[case(false, 0)]
#[case(true, 1)]
#[tokio::test]
async fn test_config_propagator_callbacks(
    #[case] register_callback: bool,
    #[case] expected: usize,
) {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let propagator = if register_callback {
        ConfigPropagator::new().on_config_change(Box::new(move |_config| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }))
    } else {
        ConfigPropagator::new()
    };

    assert_eq!(propagator.callbacks.len(), expected);
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
