//! Log event forwarder
//!
//! Bridges the synchronous tracing Layer with the async event bus.
//! Spawns a background task that drains the mpsc channel and publishes
//! `DomainEvent::LogEvent` to the event bus for SSE delivery.

use std::sync::Arc;

use mcb_domain::ports::EventBusProvider;

use super::LogEventReceiver;

/// Spawn a background task that forwards log events to the event bus.
///
/// This must be called after the event bus is available (post-DI initialization).
/// The task runs until the sender half is dropped (i.e., the tracing subscriber
/// is dropped, which effectively means process shutdown).
pub fn spawn_log_forwarder(
    mut receiver: LogEventReceiver,
    event_bus: Arc<dyn EventBusProvider>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            // Fire-and-forget: silently ignore publish errors to avoid recursion.
            // The event bus Layer itself uses a re-entrancy guard, but the
            // publish_event call may still log internally — those logs will be
            // correctly skipped by the guard.
            let _ = event_bus.publish_event(event).await;
        }
    })
}
