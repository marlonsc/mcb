use mcb_domain::events::DomainEvent;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_providers::events::TokioEventBusProvider;
use rstest::rstest;

#[tokio::test]
async fn test_event_bus_clone() {
    let bus = TokioEventBusProvider::new();
    let cloned = bus.clone();

    // Initially no subscribers
    assert!(!bus.has_subscribers());
    assert!(!cloned.has_subscribers());

    // Add a subscriber using subscribe_events which actually creates a receiver
    // The subscribe(topic) method in this impl returns an ID but doesn't change receiver count
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
    let debug = format!("{:?}", bus);
    assert!(debug.contains("TokioEventBusProvider"));
}

#[tokio::test]
async fn test_publish_event_no_subscribers() {
    let bus = TokioEventBusProvider::new();
    let event = DomainEvent::IndexingStarted {
        collection: "test".to_string(),
        total_files: 5,
    };
    let result = bus.publish_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_creates_id() {
    let bus = TokioEventBusProvider::new();
    let id = bus.subscribe("test-topic").await.unwrap();
    assert!(id.contains("tokio-broadcast-test-topic-"));
}

#[tokio::test]
async fn test_publish_invalid_payload() {
    let bus = TokioEventBusProvider::new();
    let result = bus.publish("topic", b"not-valid-json").await;
    assert!(result.is_ok());
}
