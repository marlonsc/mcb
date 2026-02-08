use mcb_domain::events::DomainEvent;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_infrastructure::infrastructure::TokioBroadcastEventBus;

#[test]
fn test_event_bus_creation() {
    let bus = TokioBroadcastEventBus::new();
    assert!(!bus.has_subscribers());
}

#[test]
fn test_event_bus_with_capacity() {
    let bus = TokioBroadcastEventBus::with_capacity(100);
    assert!(!bus.has_subscribers());
}

#[test]
fn test_event_bus_default() {
    let bus = TokioBroadcastEventBus::default();
    assert!(!bus.has_subscribers());
}

#[test]
fn test_event_bus_debug() {
    let bus = TokioBroadcastEventBus::new();
    let debug = format!("{:?}", bus);
    assert!(debug.contains("TokioBroadcastEventBus"));
}

#[test]
fn test_event_bus_clone() {
    let bus = TokioBroadcastEventBus::new();
    let cloned = bus.clone();

    // Verify cloned instance shares the same channel capacity/state or at least exists
    assert!(cloned.has_subscribers());
    assert!(!bus.has_subscribers());
}

#[tokio::test]
async fn test_publish_event_no_subscribers() {
    let bus = TokioBroadcastEventBus::new();
    let event = DomainEvent::IndexingStarted {
        collection: "test".to_string(),
        total_files: 5,
    };
    let result = bus.publish_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_creates_id() {
    let bus = TokioBroadcastEventBus::new();
    let id = bus.subscribe("test-topic").await.unwrap();
    assert!(id.contains("tokio-broadcast-test-topic-"));
}

#[tokio::test]
async fn test_publish_invalid_payload() {
    let bus = TokioBroadcastEventBus::new();
    let result = bus.publish("topic", b"not-valid-json").await;
    assert!(result.is_ok());
}
