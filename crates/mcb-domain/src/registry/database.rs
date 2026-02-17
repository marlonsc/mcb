//! Database Provider Registry
//!
//! Auto-registration system for database providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;

/// Configuration for database provider creation
///
/// Contains all configuration options that a database provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct DatabaseProviderConfig {
    /// Provider name (e.g., "sqlite", "postgres")
    pub provider: String,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(DatabaseProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::DatabaseProvider,
    config_type: DatabaseProviderConfig,
    entry_type: DatabaseProviderEntry,
    slice_name: DATABASE_PROVIDERS,
    resolve_fn: resolve_database_provider,
    list_fn: list_database_providers
);
