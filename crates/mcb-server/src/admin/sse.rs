//! Server-Sent Events (SSE) Handler
//!
//! Provides real-time event streaming for the admin dashboard.
//! Events are received from the TokioBroadcastEventBus and forwarded
//! to connected SSE clients.
//!
//! ## Supported Events
//!
//! | Event Type | Description |
//! |------------|-------------|
//! | `ConfigReloaded` | Configuration was hot-reloaded |
//! | `ServiceStateChanged` | Service lifecycle state changed |
//! | `IndexingProgress` | Indexing operation progress update |
//! | `IndexingCompleted` | Indexing operation completed |
//! | `HealthCheckCompleted` | Health check cycle completed |
//! | `MetricsSnapshot` | Periodic metrics snapshot |
//!
//! ## Usage
//!
//! Connect to `/admin/events` with an EventSource client:
//!
//! ```javascript
//! const events = new EventSource('/admin/events');
//! events.addEventListener('ServiceStateChanged', (e) => {
//!     console.log('Service state:', JSON.parse(e.data));
//! });
//! ```

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use mcb_domain::events::DomainEvent;
use std::{convert::Infallible, time::Duration};
use tracing::{debug, warn};

use super::handlers::AdminState;

/// SSE event stream handler
///
/// Streams domain events to connected clients in real-time.
/// Uses the EventBusProvider's subscribe_events() method to receive events.
pub async fn events_stream(
    State(state): State<AdminState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let event_bus = state.event_bus.clone();

    let stream = async_stream::stream! {
        // Subscribe to domain events
        let mut event_stream = match event_bus.subscribe_events().await {
            Ok(stream) => stream,
            Err(e) => {
                warn!("Failed to subscribe to events: {}", e);
                // Yield an error event and exit
                yield Ok(Event::default()
                    .event("error")
                    .data(format!("Failed to subscribe: {}", e)));
                return;
            }
        };

        debug!("SSE client connected, streaming events");

        // Stream events to the client
        use futures::StreamExt;
        while let Some(event) = event_stream.next().await {
            let event_name = get_event_name(&event);
            let event_data = match serde_json::to_string(&event) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Failed to serialize event: {}", e);
                    continue;
                }
            };

            debug!("Sending SSE event: {}", event_name);
            yield Ok(Event::default()
                .event(event_name)
                .data(event_data));
        }

        debug!("SSE event stream closed");
    };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}

/// Get the event name string for SSE event type header
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
    }
}
