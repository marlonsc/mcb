//! Event Bus Provider Implementations
//!
//! Provides event bus backends for domain events.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! | ---------- | ------ | ------------- |
//! | TokioEventBusProvider | In-Process | Tokio broadcast channels |
//! | NatsEventBusProvider | Distributed | NATS for multi-process systems |
//!
//! ## Provider Selection Guide
//!
//! - **Single Instance**: Use `TokioEventBusProvider` for in-process events
//! - **Distributed**: Use `NatsEventBusProvider` for multi-process/node systems

pub mod nats;
pub mod tokio;

// Re-export providers
// Re-export domain event types
pub use mcb_domain::events::DomainEvent;
// Re-export port trait from application layer
pub use mcb_domain::ports::infrastructure::{DomainEventStream, EventBusProvider};
pub use nats::NatsEventBusProvider;
pub use tokio::TokioEventBusProvider;
