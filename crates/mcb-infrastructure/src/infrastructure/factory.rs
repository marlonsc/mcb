//! Infrastructure factory helpers.
use std::sync::Arc;

use mcb_domain::ports::EventBusProvider;

use crate::events::BroadcastEventBus;

/// Create the default in-process event bus.
#[must_use]
pub fn default_event_bus() -> Arc<dyn EventBusProvider> {
    Arc::new(BroadcastEventBus::new())
}
