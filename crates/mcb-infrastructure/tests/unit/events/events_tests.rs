//! Trait-level tests for `EventBusProvider`
//!
//! Tests exercise the trait contract via `default_event_bus()`.
//! Concrete implementation tests live in `mcb-providers/tests/unit/events/`.

use mcb_domain::events::DomainEvent;
use mcb_infrastructure::infrastructure::default_event_bus;

#[tokio::test]
async fn test_publish_event_no_subscribers() {
    let bus = default_event_bus();
    let event = DomainEvent::IndexingStarted {
        collection: "test".to_owned(),
        total_files: 5,
    };
    let result = bus.publish_event(event).await;
    result.expect("publish_event with no subscribers should succeed");
}

#[tokio::test]
async fn test_publish_invalid_payload() {
    let bus = default_event_bus();
    let result = bus.publish("topic", b"not-valid-json").await;
    result.expect("publish with invalid payload should succeed gracefully");
}
