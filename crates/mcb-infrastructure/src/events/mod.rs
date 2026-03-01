//! In-process domain event bus.

/// Broadcast-channel event bus.
mod broadcast;

pub use broadcast::BroadcastEventBus;
