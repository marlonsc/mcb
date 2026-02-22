//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Server-Sent Events (SSE) Handler
//!
//! Provides real-time event streaming for the admin dashboard.
//! Events are received from the `TokioBroadcastEventBus` and forwarded
//! to connected SSE clients.
//!
//! ## Supported Events
//!
//! | Event Type | Description |
//! | ------------ | ------------- |
//! | `ConfigReloaded` | Configuration was hot-reloaded |
//! | `ServiceStateChanged` | Service lifecycle state changed |
//! | `IndexingProgress` | Indexing operation progress update |
//! | `IndexingCompleted` | Indexing operation completed |
//! | `ValidationStarted` | Validation operation started |
//! | `ValidationProgress` | Validation operation progress update |
//! | `ValidationCompleted` | Validation operation completed |
//! | `HealthCheckCompleted` | Health check cycle completed |
//! | `MetricsSnapshot` | Periodic metrics snapshot |
//! | `LogEvent` | Server log event (warn/error by default) |
//!
//! ## Usage
//!
//! Connect to `/admin/events` with an `EventSource` client:
//!
//! ```javascript
//! const events = new EventSource('/admin/events');
//! events.addEventListener('ServiceStateChanged', (e) => {
//!     console.log('Service state:', JSON.parse(e.data));
//! });
//! ```

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::StreamExt;
use mcb_domain::events::DomainEvent;
use mcb_domain::{debug, info, warn};

use super::handlers::AdminState;

/// SSE event stream handler
///
/// Streams domain events to connected clients in real-time.
/// Uses the `EventBusProvider`'s `subscribe_events()` method to receive events.
pub async fn events_stream(
    State(state): State<Arc<AdminState>>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    info!("sse", "events_stream called");
    let event_bus = Arc::clone(&state.event_bus);

    let stream = async_stream::stream! {
        // Subscribe to domain events
        let mut event_stream = match event_bus.subscribe_events().await {
            Ok(stream) => stream,
            Err(e) => {
                warn!("sse", "Failed to subscribe to events", &e);
                // Yield an error event and exit
                yield Ok(Event::default()
                    .event("error")
                    .data(format!("Failed to subscribe: {e}")));
                return;
            }
        };

        debug!("sse", "SSE client connected, streaming events");

        // Stream events to the client
        while let Some(event) = event_stream.next().await {
            let event_name = get_event_name(&event);
            let event_data = match serde_json::to_string(&event) {
                Ok(data) => data,
                Err(e) => {
                    warn!("sse", "Failed to serialize event", &e);
                    continue;
                }
            };

            debug!("sse", "Sending SSE event", &event_name);
            yield Ok(Event::default().event(event_name).data(event_data));
        }

        debug!("sse", "SSE event stream closed");
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Get the event name string for SSE event type header
#[must_use]
pub fn get_event_name(event: &DomainEvent) -> &'static str {
    match event {
        // Indexing events
        DomainEvent::IndexRebuild { .. } => "IndexRebuild",
        DomainEvent::IndexingStarted { .. } => "IndexingStarted",
        DomainEvent::IndexingProgress { .. } => "IndexingProgress",
        DomainEvent::IndexingCompleted { .. } => "IndexingCompleted",
        // Sync events
        DomainEvent::SyncCompleted { .. } => "SyncCompleted",
        // Cache events
        DomainEvent::CacheInvalidate { .. } => "CacheInvalidate",
        // Snapshot events
        DomainEvent::SnapshotCreated { .. } => "SnapshotCreated",
        // File watcher events
        DomainEvent::FileChangesDetected { .. } => "FileChangesDetected",
        // Service lifecycle events
        DomainEvent::ServiceStateChanged { .. } => "ServiceStateChanged",
        // Configuration events
        DomainEvent::ConfigReloaded { .. } => "ConfigReloaded",
        // Health events
        DomainEvent::HealthCheckCompleted { .. } => "HealthCheckCompleted",
        // Metrics events
        DomainEvent::MetricsSnapshot { .. } => "MetricsSnapshot",
        // Search events
        DomainEvent::SearchExecuted { .. } => "SearchExecuted",
        // Validation events
        DomainEvent::ValidationStarted { .. } => "ValidationStarted",
        DomainEvent::ValidationProgress { .. } => "ValidationProgress",
        DomainEvent::ValidationCompleted { .. } => "ValidationCompleted",
        DomainEvent::LogEvent { .. } => "LogEvent",
    }
}
