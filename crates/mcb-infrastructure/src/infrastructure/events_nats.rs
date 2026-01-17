//! NATS EventBus Implementation
//!
//! Provides a distributed event bus using NATS for multi-process/distributed systems.
//!
//! ## ARCHITECTURE RULE
//!
//! This implementation is **INTERNAL** to the infrastructure layer.
//! External code MUST resolve `Arc<dyn EventBusProvider>` via Shaku DI.
//! NEVER import or use this type directly.
//!
//! ## Usage
//!
//! Configure via `config.toml`:
//! ```toml
//! [system.infrastructure.event_bus]
//! provider = "nats"
//! nats_url = "nats://localhost:4222"
//! ```

use async_nats::Client;
use async_trait::async_trait;
use futures::stream;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::{Error, Result};
use mcb_domain::events::DomainEvent;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Default subject for domain events
const DEFAULT_SUBJECT: &str = "mcb.events";

/// NATS EventBus for distributed systems
///
/// This implementation is INTERNAL. Resolve via Shaku DI as `Arc<dyn EventBusProvider>`.
pub struct NatsEventBus {
    client: Arc<Client>,
    subject: String,
    /// Active subscribers for tracking
    subscriber_count: Arc<RwLock<usize>>,
}

impl NatsEventBus {
    /// Create a new NATS EventBus
    ///
    /// # Arguments
    /// * `url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Errors
    /// Returns an error if connection to NATS server fails
    pub async fn new(url: &str) -> Result<Self> {
        Self::with_subject(url, DEFAULT_SUBJECT).await
    }

    /// Create a new NATS EventBus with custom subject
    pub async fn with_subject(url: &str, subject: &str) -> Result<Self> {
        info!("Connecting to NATS server at {}", url);

        let client = async_nats::connect(url).await.map_err(|e| Error::Provider {
            message: format!("Failed to connect to NATS server at {}: {}", url, e),
        })?;

        info!("Connected to NATS server at {}", url);

        Ok(Self {
            client: Arc::new(client),
            subject: subject.to_string(),
            subscriber_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Create a new NATS EventBus with client options
    pub async fn with_options(
        url: &str,
        subject: &str,
        client_name: Option<&str>,
    ) -> Result<Self> {
        info!("Connecting to NATS server at {} with options", url);

        let mut options = async_nats::ConnectOptions::new();

        if let Some(name) = client_name {
            options = options.name(name);
        }

        let client = options.connect(url).await.map_err(|e| Error::Provider {
            message: format!("Failed to connect to NATS server at {}: {}", url, e),
        })?;

        info!("Connected to NATS server at {}", url);

        Ok(Self {
            client: Arc::new(client),
            subject: subject.to_string(),
            subscriber_count: Arc::new(RwLock::new(0)),
        })
    }
}

impl std::fmt::Debug for NatsEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsEventBus")
            .field("subject", &self.subject)
            .finish()
    }
}

#[async_trait]
impl EventBusProvider for NatsEventBus {
    // ========================================================================
    // High-Level Typed API
    // ========================================================================

    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        let payload = serde_json::to_vec(&event).map_err(|e| Error::Provider {
            message: format!("Failed to serialize event: {}", e),
        })?;

        self.client
            .publish(self.subject.clone(), payload.into())
            .await
            .map_err(|e| Error::Provider {
                message: format!("Failed to publish event to NATS: {}", e),
            })?;

        debug!("Published event to NATS subject '{}'", self.subject);
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        let subscriber = self
            .client
            .subscribe(self.subject.clone())
            .await
            .map_err(|e| Error::Provider {
                message: format!("Failed to subscribe to NATS subject '{}': {}", self.subject, e),
            })?;

        // Increment subscriber count
        {
            let mut count = self.subscriber_count.write().await;
            *count += 1;
        }

        let subscriber_count = Arc::clone(&self.subscriber_count);

        // Convert NATS messages to DomainEvent stream
        let stream = stream::unfold(
            (subscriber, subscriber_count),
            |(mut sub, count)| async move {
                match sub.next().await {
                    Some(msg) => {
                        match serde_json::from_slice::<DomainEvent>(&msg.payload) {
                            Ok(event) => Some((event, (sub, count))),
                            Err(e) => {
                                warn!("Failed to deserialize NATS message: {}", e);
                                // Skip bad messages and continue
                                Some((
                                    DomainEvent::MetricsSnapshot {
                                        timestamp: chrono::Utc::now(),
                                    },
                                    (sub, count),
                                ))
                            }
                        }
                    }
                    None => {
                        // Decrement subscriber count when stream ends
                        let mut c = count.write().await;
                        *c = c.saturating_sub(1);
                        None
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    fn has_subscribers(&self) -> bool {
        // This is a best-effort check - NATS doesn't provide real-time subscriber info
        // We track local subscribers only
        match self.subscriber_count.try_read() {
            Ok(count) => *count > 0,
            Err(_) => false,
        }
    }

    // ========================================================================
    // Low-Level Raw API
    // ========================================================================

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let subject = if topic.is_empty() {
            self.subject.clone()
        } else {
            format!("{}.{}", self.subject, topic)
        };

        self.client
            .publish(subject.clone(), payload.to_vec().into())
            .await
            .map_err(|e| Error::Provider {
                message: format!("Failed to publish to NATS subject '{}': {}", subject, e),
            })?;

        debug!("Published raw payload to NATS subject '{}'", subject);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        let subject = if topic.is_empty() {
            self.subject.clone()
        } else {
            format!("{}.{}", self.subject, topic)
        };

        // Create subscription for tracking purposes
        let _sub = self
            .client
            .subscribe(subject.clone())
            .await
            .map_err(|e| Error::Provider {
                message: format!("Failed to subscribe to NATS subject '{}': {}", subject, e),
            })?;

        let sub_id = format!("nats-{}-{}", subject, uuid::Uuid::new_v4());
        debug!("Created NATS subscription: {}", sub_id);

        Ok(sub_id)
    }
}

/// Factory for creating NatsEventBus from configuration
pub struct NatsEventBusFactory;

impl NatsEventBusFactory {
    /// Create NatsEventBus from EventBusConfig
    pub async fn create(
        url: &str,
        client_name: Option<&str>,
    ) -> Result<NatsEventBus> {
        NatsEventBus::with_options(url, DEFAULT_SUBJECT, client_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running NATS server
    // Run with: docker run -p 4222:4222 nats:latest

    #[tokio::test]
    #[ignore = "Requires running NATS server"]
    async fn test_nats_connect() {
        let bus = NatsEventBus::new("nats://localhost:4222").await;
        assert!(bus.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires running NATS server"]
    async fn test_nats_publish_subscribe() {
        use futures::StreamExt;
        use mcb_domain::events::ServiceState;

        let bus = NatsEventBus::new("nats://localhost:4222").await.unwrap();

        // Subscribe first
        let mut stream = bus.subscribe_events().await.unwrap();

        // Publish event
        let event = DomainEvent::ServiceStateChanged {
            name: "test".to_string(),
            state: ServiceState::Running,
            previous_state: None,
        };

        bus.publish_event(event.clone()).await.unwrap();

        // Receive event (with timeout)
        let received = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stream.next(),
        )
        .await
        .expect("Timeout waiting for event")
        .expect("Stream ended unexpectedly");

        assert_eq!(received, event);
    }
}
