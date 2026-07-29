//! Event bus provider ports.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::error::Result;
use crate::events::DomainEvent;

/// Boxed async stream of domain events
pub type DomainEventStream = Pin<Box<dyn Stream<Item = DomainEvent> + Send + Sync + 'static>>;

/// Event bus provider interface for typed event pub/sub
#[async_trait]
pub trait EventBusProvider: Send + Sync {
    /// Publish a domain event to the bus.
    async fn publish_event(&self, event: DomainEvent) -> Result<()>;
    /// Subscribe to all domain events.
    async fn subscribe_events(&self) -> Result<DomainEventStream>;
    /// Check if there are any active subscribers.
    fn has_subscribers(&self) -> bool;

    // Low-Level Raw API
    /// Publish raw payload to a specific topic.
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
    /// Subscribe to a specific topic.
    async fn subscribe(&self, topic: &str) -> Result<String>;
}
