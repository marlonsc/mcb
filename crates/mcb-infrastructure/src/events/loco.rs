use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{DomainEventStream, EventBusProvider};
use mcb_domain::utils::id;
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::constants::events::EVENT_BUS_BUFFER_SIZE;

/// Loco-owned in-process event bus adapter backed by `tokio::broadcast`.
#[derive(Clone)]
pub struct LocoEventBusProvider {
    sender: Arc<broadcast::Sender<DomainEvent>>,
}

impl LocoEventBusProvider {
    /// Create a new event bus with default buffer size.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_BUFFER_SIZE);
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Default for LocoEventBusProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LocoEventBusProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocoEventBusProvider")
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

#[async_trait]
impl EventBusProvider for LocoEventBusProvider {
    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        match self.sender.send(event) {
            Ok(count) => debug!("Published event to {count} subscribers"),
            Err(_) => debug!("Published event but no subscribers"),
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
                        warn!("Event stream lagged by {n} events");
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
                warn!("Failed to deserialize event payload for topic '{topic}': {e}");
                return Ok(());
            }
        };
        debug!("Publishing event to topic '{topic}': {event:?}");
        self.publish_event(event).await
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        Ok(format!("loco-broadcast-{topic}-{}", id::generate()))
    }
}
