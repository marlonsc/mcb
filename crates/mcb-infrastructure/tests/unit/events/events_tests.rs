//! Trait-level tests for `EventBusProvider`
//!
//! Tests exercise the trait contract via `BroadcastEventBus`.
//! Concrete implementation tests live in `mcb-providers/tests/unit/events/`.

use mcb_domain::events::DomainEvent;
use mcb_domain::ports::EventBusProvider;
use mcb_domain::registry::events::{EventBusProviderConfig, resolve_event_bus_provider};
use mcb_domain::test_utils::TestResult;
use rstest::{fixture, rstest};
use std::sync::Arc;

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
    bus.publish_event(event).await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_publish_invalid_payload(
    event_bus: TestResult<Arc<dyn EventBusProvider>>,
) -> TestResult {
    let bus = event_bus?;
    bus.publish("topic", b"not-valid-json").await?;
    Ok(())
}
