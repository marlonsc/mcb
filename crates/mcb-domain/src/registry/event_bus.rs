#![allow(missing_docs)]

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct EventBusProviderConfig {
    pub provider: String,
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
