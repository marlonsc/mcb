use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct IndexingOperationsProviderConfig {
    pub provider: String,
    pub extra: HashMap<String, String>,
}

impl IndexingOperationsProviderConfig {
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

#[derive(Debug, Clone, Default)]
pub struct ValidationOperationsProviderConfig {
    pub provider: String,
    pub extra: HashMap<String, String>,
}

impl ValidationOperationsProviderConfig {
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
