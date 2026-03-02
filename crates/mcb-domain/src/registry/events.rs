//! Event Bus Provider Registry
//!
//! Auto-registration system for event bus providers using linkme distributed slices.

use std::collections::HashMap;

/// Configuration for event bus provider creation.
#[derive(Debug, Clone, Default)]
pub struct EventBusProviderConfig {
    /// Provider name (e.g., "inprocess")
    pub provider: String,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(EventBusProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::EventBusProvider,
    config_type: EventBusProviderConfig,
    entry_type: EventBusProviderEntry,
    slice_name: EVENT_BUS_PROVIDERS,
    resolve_fn: resolve_event_bus_provider,
    list_fn: list_event_bus_providers
);
