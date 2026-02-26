//! Trait-level tests for `EventBusProvider`
//!
//! Tests exercise the trait contract via `BroadcastEventBus`.
//! Concrete implementation tests live in `mcb-providers/tests/unit/events/`.

use mcb_domain::events::DomainEvent;
use mcb_domain::ports::EventBusProvider;
use mcb_infrastructure::events::BroadcastEventBus;
use std::sync::Arc;

#[tokio::test]
async fn test_publish_event_no_subscribers() {
    let bus: Arc<dyn EventBusProvider> = Arc::new(BroadcastEventBus::new());
    let event = DomainEvent::IndexingStarted {
        collection: "test".to_owned(),
        total_files: 5,
    };
    let result = bus.publish_event(event).await;
    result.expect("publish_event with no subscribers should succeed");
}

#[tokio::test]
async fn test_publish_invalid_payload() {
    let bus: Arc<dyn EventBusProvider> = Arc::new(BroadcastEventBus::new());
    let result = bus.publish("topic", b"not-valid-json").await;
    result.expect("publish with invalid payload should succeed gracefully");
}
