//! Infrastructure factory helpers.
use std::sync::Arc;

use mcb_domain::ports::EventBusProvider;

/// Create the default in-process event bus.
#[must_use]
pub fn default_event_bus() -> Arc<dyn EventBusProvider> {
    Arc::new(mcb_providers::events::TokioEventBusProvider::new())
}
