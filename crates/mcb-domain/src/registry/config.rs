//! Configuration Provider Registry
//!
//! Auto-registration system for config providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at link time.

use std::collections::HashMap;

/// Configuration for config provider creation.
#[derive(Debug, Clone, Default)]
pub struct ConfigProviderConfig {
    /// Provider name (e.g., "`loco_yaml`")
    pub provider: String,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(ConfigProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::ConfigProvider,
    config_type: ConfigProviderConfig,
    entry_type: ConfigProviderEntry,
    slice_name: CONFIG_PROVIDERS,
    resolve_fn: resolve_config_provider,
    list_fn: list_config_providers
);
