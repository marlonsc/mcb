use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{DomainEventStream, EventBusProvider};
use mcb_domain::utils::id;
use tokio::sync::broadcast;

use crate::constants::events::EVENT_BUS_BUFFER_SIZE;

/// In-process domain event bus backed by a broadcast channel.
#[derive(Clone)]
pub struct BroadcastEventBus {
    sender: Arc<broadcast::Sender<DomainEvent>>,
}

impl BroadcastEventBus {
    /// Create a new event bus with default buffer size.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_BUFFER_SIZE);
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Default for BroadcastEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BroadcastEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastEventBus")
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

#[async_trait]
impl EventBusProvider for BroadcastEventBus {
    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        match self.sender.send(event) {
            Ok(count) => mcb_domain::debug!(
                "event_bus",
                &format!("Published event to {count} subscribers")
            ),
            Err(_) => mcb_domain::debug!("event_bus", "Published event but no subscribers"),
        }
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        let receiver = self.sender.subscribe();
        let stream = stream::unfold(receiver, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => return Some((event, rx)),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        mcb_domain::warn!(
                            "event_bus",
                            "Event stream lagged",
                            &format!("{n} events")
                        );
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        });
        Ok(Box::pin(stream))
    }

    fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let event: DomainEvent = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                mcb_domain::warn!(
                    "event_bus",
                    "Failed to deserialize event payload for topic",
                    &format!("topic={topic} error={e}")
                );
                return Ok(());
            }
        };
        mcb_domain::debug!(
            "event_bus",
            &format!("Publishing event to topic '{topic}': {event:?}")
        );
        self.publish_event(event).await
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        Ok(format!("broadcast-{topic}-{}", id::generate()))
    }
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::events::{EVENT_BUS_PROVIDERS, EventBusProviderEntry};

// linkme distributed_slice uses unsafe link-section attributes internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(EVENT_BUS_PROVIDERS)]
static BROADCAST_EVENT_BUS_ENTRY: EventBusProviderEntry = EventBusProviderEntry {
    name: "inprocess",
    description: "In-process broadcast channel event bus",
    build: |_config| Ok(Arc::new(BroadcastEventBus::new())),
};
