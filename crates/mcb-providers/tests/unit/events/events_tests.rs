//! Concrete-type tests for `TokioEventBusProvider`
//!
//! Tests specific to the Tokio broadcast implementation (clone behavior,
//! capacity, debug output, subscription ID format).

use mcb_domain::ports::EventBusProvider;
use mcb_providers::events::TokioEventBusProvider;
use rstest::rstest;

#[tokio::test]
async fn test_event_bus_clone() {
    let bus = TokioEventBusProvider::new();
    let cloned = bus.clone();

    assert!(!bus.has_subscribers());
    assert!(!cloned.has_subscribers());

    let _stream = bus.subscribe_events().await.unwrap();

    // Both should see the subscriber because they share the same channel
    assert!(bus.has_subscribers());
    assert!(cloned.has_subscribers());
}

#[rstest]
#[case(false)]
#[case(true)]
fn event_bus_initial_subscriber_state(#[case] use_default: bool) {
    let bus = if use_default {
        TokioEventBusProvider::default()
    } else {
        TokioEventBusProvider::with_capacity(100)
    };
    assert!(!bus.has_subscribers());
}

#[rstest]
fn test_event_bus_debug() {
    let bus = TokioEventBusProvider::new();
    let debug = format!("{bus:?}");
    assert!(debug.contains("TokioEventBusProvider"));
}

#[tokio::test]
async fn test_subscribe_creates_id() {
    let bus = TokioEventBusProvider::new();
    let id = bus.subscribe("test-topic").await.unwrap();
    assert!(id.contains("tokio-broadcast-test-topic-"));
}
