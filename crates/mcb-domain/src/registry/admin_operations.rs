use std::collections::HashMap;

/// Configuration for indexing operations providers.
#[derive(Debug, Clone, Default)]
pub struct IndexingOperationsProviderConfig {
    /// Provider implementation name.
    pub provider: String,
    /// Extra configuration parameters.
    pub extra: HashMap<String, String>,
}

impl IndexingOperationsProviderConfig {
    /// Create a new configuration for the given provider.
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Self::default()
        }
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::IndexingOperationsInterface,
    config_type: IndexingOperationsProviderConfig,
    entry_type: IndexingOperationsProviderEntry,
    slice_name: INDEXING_OPERATIONS_PROVIDERS,
    resolve_fn: resolve_indexing_operations_provider,
    list_fn: list_indexing_operations_providers
);

/// Configuration for validation operations providers.
#[derive(Debug, Clone, Default)]
pub struct ValidationOperationsProviderConfig {
    /// Provider implementation name.
    pub provider: String,
    /// Extra configuration parameters.
    pub extra: HashMap<String, String>,
}

impl ValidationOperationsProviderConfig {
    /// Create a new configuration for the given provider.
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Self::default()
        }
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::ValidationOperationsInterface,
    config_type: ValidationOperationsProviderConfig,
    entry_type: ValidationOperationsProviderEntry,
    slice_name: VALIDATION_OPERATIONS_PROVIDERS,
    resolve_fn: resolve_validation_operations_provider,
    list_fn: list_validation_operations_providers
);
