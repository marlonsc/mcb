//! Unit tests for domain events

use rstest::rstest;
use std::sync::Mutex;

use async_trait::async_trait;
use mcb_domain::events::{DomainEvent, EventPublisher};

// Mock event publisher for testing
struct TestEventPublisher {
    published_events: Mutex<Vec<DomainEvent>>,
    subscriber_count: usize,
}

impl TestEventPublisher {
    fn new() -> Self {
        Self {
            published_events: Mutex::new(Vec::new()),
            subscriber_count: 1,
        }
    }

    fn with_no_subscribers() -> Self {
        Self {
            published_events: Mutex::new(Vec::new()),
            subscriber_count: 0,
        }
    }

    fn get_published_events(&self) -> Vec<DomainEvent> {
        match self.published_events.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

#[async_trait]
impl EventPublisher for TestEventPublisher {
    async fn publish(&self, event: DomainEvent) -> mcb_domain::Result<()> {
        match self.published_events.lock() {
            Ok(mut guard) => guard.push(event),
            Err(mut poisoned) => poisoned.get_mut().push(event),
        }
        Ok(())
    }

    fn has_subscribers(&self) -> bool {
        self.subscriber_count > 0
    }
}

#[rstest]
#[case(DomainEvent::IndexRebuild { collection: Some("test-collection".to_owned()) }, "IndexRebuild")]
#[case(
    DomainEvent::SyncCompleted { path: "/path/to/code".to_owned(), files_changed: 42 },
    "SyncCompleted"
)]
#[case(
    DomainEvent::CacheInvalidate { namespace: Some("embeddings".to_owned()) },
    "CacheInvalidate"
)]
#[case(
    DomainEvent::SnapshotCreated { root_path: "/code".to_owned(), file_count: 100 },
    "SnapshotCreated"
)]
#[case(
    DomainEvent::FileChangesDetected { root_path: "/code".to_owned(), added: 5, modified: 10, removed: 2 },
    "FileChangesDetected"
)]
#[case(
    DomainEvent::LogEvent { level: "WARN".to_owned(), message: "test warning".to_owned(), target: "mcb_server::test".to_owned(), timestamp: 1700000000000 },
    "LogEvent"
)]
fn domain_event_variants(#[case] event: DomainEvent, #[case] expected_debug_fragment: &str) {
    let debug_str = format!("{event:?}");
    assert!(debug_str.contains(expected_debug_fragment));
}

#[test]
fn test_domain_event_clone() {
    let event1 = DomainEvent::SyncCompleted {
        path: "/code".to_owned(),
        files_changed: 10,
    };

    let event2 = event1.clone();

    assert_eq!(event1, event2);
}

#[test]
fn test_event_publisher_creation() {
    let publisher = TestEventPublisher::new();
    let events = publisher.get_published_events();
    assert!(events.is_empty());
}

#[rstest]
#[case(true)]
#[case(false)]
fn has_subscribers(#[case] expected_has_subscribers: bool) {
    let publisher = if expected_has_subscribers {
        TestEventPublisher::new()
    } else {
        TestEventPublisher::with_no_subscribers()
    };
    assert_eq!(publisher.has_subscribers(), expected_has_subscribers);
}

#[rstest]
#[case(vec![DomainEvent::IndexRebuild { collection: Some("test".to_owned()) }], 1)]
#[case(
    vec![
        DomainEvent::IndexRebuild { collection: Some("coll-1".to_owned()) },
        DomainEvent::SyncCompleted { path: "/path".to_owned(), files_changed: 5 },
        DomainEvent::CacheInvalidate { namespace: None },
    ],
    3
)]
#[tokio::test]
async fn publish_events(#[case] events: Vec<DomainEvent>, #[case] expected_len: usize) {
    let publisher = TestEventPublisher::new();

    for event in events {
        let result = publisher.publish(event).await;
        assert!(result.is_ok());
    }

    let published_events = publisher.get_published_events();
    assert_eq!(published_events.len(), expected_len);

    if expected_len == 1 {
        assert!(matches!(
            &published_events[0],
            DomainEvent::IndexRebuild { collection } if collection == &Some("test".to_owned())
        ));
    }
}

#[test]
fn test_event_publisher_trait_object() {
    // Test that we can use EventPublisher as a trait object
    let publisher: Box<dyn EventPublisher> = Box::new(TestEventPublisher::new());
    assert!(publisher.has_subscribers());
}

#[tokio::test]
async fn test_event_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Events should be serializable (for transport/logging)
    let event = DomainEvent::FileChangesDetected {
        root_path: "/code".to_owned(),
        added: 1,
        modified: 2,
        removed: 3,
    };

    let json = serde_json::to_string(&event)?;
    assert!(json.contains("FileChangesDetected"));
    assert!(json.contains("/code"));

    let deserialized: DomainEvent = serde_json::from_str(&json)?;
    assert_eq!(event, deserialized);
    Ok(())
}
