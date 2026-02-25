//! In-process domain event bus.

/// Broadcast-channel event bus.
pub mod broadcast;

pub use broadcast::BroadcastEventBus;
