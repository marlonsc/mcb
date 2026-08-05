//! Trait-level tests for `EventBusProvider`
//!
//! Tests exercise the trait contract via `BroadcastEventBus`.
//! Concrete implementation tests live in `mcb-providers/tests/unit/events/`.

use futures::StreamExt;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::EventBusProvider;
use mcb_domain::registry::events::{EventBusProviderConfig, resolve_event_bus_provider};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::{fixture, rstest};
use std::sync::Arc;
use std::time::Duration;

#[fixture]
fn event_bus() -> TestResult<Arc<dyn EventBusProvider>> {
    resolve_event_bus_provider(&EventBusProviderConfig::new("inprocess")).map_err(Into::into)
}

#[rstest]
#[tokio::test]
async fn test_publish_event_no_subscribers(
    event_bus: TestResult<Arc<dyn EventBusProvider>>,
) -> TestResult {
    let bus = event_bus?;
    let event = DomainEvent::IndexingStarted {
        collection: "test".to_owned(),
        total_files: 5,
    };
    let mut stream = bus.subscribe_events().await?;
    assert!(
        bus.has_subscribers(),
        "subscription should register a receiver"
    );
    bus.publish_event(event).await?;
    let received = tokio::time::timeout(Duration::from_secs(1), stream.next()).await?;
    assert_eq!(
        received,
        Some(DomainEvent::IndexingStarted {
            collection: "test".to_owned(),
            total_files: 5,
        })
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_publish_invalid_payload(
    event_bus: TestResult<Arc<dyn EventBusProvider>>,
) -> TestResult {
    let bus = event_bus?;
    let mut stream = bus.subscribe_events().await?;
    bus.publish("topic", b"not-valid-json").await?;
    let received = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert!(
        received.is_err(),
        "invalid raw payload must not publish a domain event"
    );
    Ok(())
}
