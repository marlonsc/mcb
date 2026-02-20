//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Custom tracing Layer that forwards log events to the event bus.
//!
//! Captures tracing events and sends them as `DomainEvent::LogEvent` through
//! an unbounded mpsc channel. A background task (see [`super::forwarder`])
//! drains the channel and publishes events to the event bus for SSE delivery.
//!
//! ## Re-entrancy Protection
//!
//! A thread-local guard prevents infinite recursion when the event bus or
//! forwarder task itself produces log output.

use std::cell::Cell;
use std::fmt::Write;

use mcb_domain::events::DomainEvent;
use tokio::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

use super::LogEventReceiver;

thread_local! {
    /// Re-entrancy guard — prevents the layer from processing its own log output
    static GUARD: Cell<bool> = const { Cell::new(false) };
}

/// Tracing layer that captures events and sends them to an mpsc channel
pub struct EventBusLayer {
    /// Channel sender (non-blocking, sync-safe)
    sender: mpsc::UnboundedSender<DomainEvent>,
    /// Minimum level to capture
    min_level: Level,
}

impl EventBusLayer {
    /// Create a new event bus layer with the specified minimum level.
    ///
    /// Returns the layer and a receiver that must be connected to the event bus.
    pub fn new(min_level: Level) -> (Self, LogEventReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let layer = Self { sender, min_level };
        (layer, receiver)
    }
}

impl<S: Subscriber> Layer<S> for EventBusLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Skip if below minimum level
        if event.metadata().level() > &self.min_level {
            return;
        }

        // Re-entrancy guard: skip if we're already inside this layer
        let is_reentrant = GUARD.with(|g| {
            if g.get() {
                return true;
            }
            g.set(true);
            false
        });
        if is_reentrant {
            return;
        }

        // Extract fields from the event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        // Prefer log.target (from log-to-tracing bridge) over generic "log" target
        let target = visitor
            .log_target
            .unwrap_or_else(|| event.metadata().target().to_owned());

        let domain_event = DomainEvent::LogEvent {
            level: event.metadata().level().to_string(),
            message: visitor.message,
            target,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Non-blocking send — drop events if the channel is closed
        let _ = self.sender.send(domain_event);

        // Release the re-entrancy guard
        GUARD.with(|g| g.set(false));
    }
}

/// Visitor that extracts the message, structured fields, and log bridge target
#[derive(Default)]
struct MessageVisitor {
    /// Accumulated message string
    message: String,
    /// Original target from log-to-tracing bridge (more specific than "log")
    log_target: Option<String>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else if !is_internal_field(field.name()) {
            if !self.message.is_empty() {
                let _ = write!(self.message, " {}={:?}", field.name(), value);
            } else {
                let _ = write!(self.message, "{}={:?}", field.name(), value);
            }
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        } else if field.name() == "log.target" {
            self.log_target = Some(value.to_owned());
        } else if !is_internal_field(field.name()) {
            if !self.message.is_empty() {
                let _ = write!(self.message, " {}={}", field.name(), value);
            } else {
                let _ = write!(self.message, "{}={}", field.name(), value);
            }
        }
    }
}

/// Internal tracing/log-bridge metadata fields to exclude from messages
fn is_internal_field(name: &str) -> bool {
    matches!(
        name,
        "log.target" | "log.module_path" | "log.file" | "log.line"
    )
}
