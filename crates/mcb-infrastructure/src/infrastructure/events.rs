//! Event Bus Adapters
//!
//! Provides event bus implementations for publishing and subscribing to domain events.
//!
//! ## Available Implementations
//!
//! | Provider | Use Case |
//! |----------|----------|
//! | [`TokioBroadcastEventBus`] | Production (single process, high performance) |
//! | [`NullEventBus`] | Testing (no-op) |
//!
//! ## Production Default
//!
//! `TokioBroadcastEventBus` is the default for production use. It provides:
//! - In-process event broadcasting using Tokio's broadcast channel
//! - Zero-copy message passing
//! - Backpressure handling via configurable buffer capacity
//! - Direct `DomainEvent` subscription for SSE/WebSocket streaming

use async_trait::async_trait;
use mcb_application::ports::infrastructure::EventBusProvider;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Default buffer capacity for the broadcast channel
const DEFAULT_CAPACITY: usize = 1024;

/// Production EventBus using Tokio broadcast channel
///
/// This implementation provides high-performance in-process event broadcasting.
/// Use this as the default for single-process deployments.
///
/// ## Features
///
/// - Zero external dependencies
/// - Direct `DomainEvent` subscription for type-safe event handling
/// - Configurable buffer capacity
/// - Graceful handling of slow subscribers (lagged events are dropped)
///
/// ## Example
///
/// ```ignore
/// use mcb_infrastructure::infrastructure::events::TokioBroadcastEventBus;
///
/// let bus = TokioBroadcastEventBus::new();
/// let mut receiver = bus.subscribe_events();
///
/// // Publish via EventBusProvider trait
/// bus.publish("events.domain", &serialized_event).await?;
///
/// // Or receive typed events directly
/// while let Ok(event) = receiver.recv().await {
///     match event {
///         DomainEvent::IndexingProgress { .. } => { /* handle */ }
///         _ => {}
///     }
/// }
/// ```
#[derive(Clone)]
pub struct TokioBroadcastEventBus {
    sender: Arc<broadcast::Sender<DomainEvent>>,
    capacity: usize,
}

impl TokioBroadcastEventBus {
    /// Create a new event bus with default capacity (1024 events)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
            capacity,
        }
    }

    /// Subscribe to receive typed `DomainEvent` directly
    ///
    /// This is the preferred method for SSE/WebSocket handlers that need
    /// type-safe access to domain events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }

    /// Publish a typed `DomainEvent` directly
    ///
    /// Returns the number of receivers that received the event.
    pub fn publish_event(&self, event: DomainEvent) -> usize {
        match self.sender.send(event) {
            Ok(count) => {
                debug!("Published event to {} subscribers", count);
                count
            }
            Err(_) => {
                // No receivers - not an error, just no one listening
                debug!("Published event but no subscribers");
                0
            }
        }
    }

    /// Check if there are any active subscribers
    pub fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }

    /// Get the buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for TokioBroadcastEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TokioBroadcastEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioBroadcastEventBus")
            .field("capacity", &self.capacity)
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

// Shaku Component implementation for DI container
impl<M: shaku::Module> shaku::Component<M> for TokioBroadcastEventBus {
    type Interface = dyn EventBusProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(TokioBroadcastEventBus::new())
    }
}

#[async_trait]
impl EventBusProvider for TokioBroadcastEventBus {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        // Deserialize payload to DomainEvent
        let event: DomainEvent = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to deserialize event payload: {}", e);
                return Ok(()); // Don't fail on bad payload
            }
        };

        debug!("Publishing event to topic '{}': {:?}", topic, event);
        self.publish_event(event);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        // Note: actual subscription is done via subscribe_events()
        // This just returns an ID for tracking
        let id = format!("tokio-broadcast-{}-{}", topic, uuid::Uuid::new_v4());
        debug!("Created subscription: {}", id);
        Ok(id)
    }
}

// ============================================================================
// Null Implementation (Testing)
// ============================================================================

/// Null implementation for testing
///
/// This provider does nothing - all operations succeed but no events are delivered.
/// Use for unit tests where event delivery is not being tested.
#[derive(shaku::Component)]
#[shaku(interface = EventBusProvider)]
pub struct NullEventBus;

impl NullEventBus {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBusProvider for NullEventBus {
    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("null-sub".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcb_domain::events::ServiceState;

    #[tokio::test]
    async fn test_tokio_broadcast_publish_subscribe() {
        let bus = TokioBroadcastEventBus::new();
        let mut receiver = bus.subscribe_events();

        // Publish an event
        let event = DomainEvent::ServiceStateChanged {
            name: "test-service".to_string(),
            state: ServiceState::Running,
            previous_state: Some(ServiceState::Starting),
        };

        let count = bus.publish_event(event.clone());
        assert_eq!(count, 1);

        // Receive the event
        let received = receiver.recv().await.unwrap();
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn test_tokio_broadcast_no_subscribers() {
        let bus = TokioBroadcastEventBus::new();

        // No subscribers - should not panic
        let event = DomainEvent::MetricsSnapshot {
            timestamp: chrono::Utc::now(),
        };
        let count = bus.publish_event(event);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_tokio_broadcast_via_trait() {
        let bus = TokioBroadcastEventBus::new();
        let mut receiver = bus.subscribe_events();

        let event = DomainEvent::IndexingProgress {
            collection: "test".to_string(),
            processed: 50,
            total: 100,
            current_file: Some("main.rs".to_string()),
        };

        // Publish via EventBusProvider trait
        let payload = serde_json::to_vec(&event).unwrap();
        bus.publish("events.indexing", &payload).await.unwrap();

        // Receive typed event
        let received = receiver.recv().await.unwrap();
        assert_eq!(received, event);
    }

    #[test]
    fn test_null_event_bus() {
        let bus = NullEventBus::new();
        assert!(tokio_test::block_on(bus.publish("test", b"data")).is_ok());
        assert!(tokio_test::block_on(bus.subscribe("test")).is_ok());
    }
}
