#![allow(missing_docs)]

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FileSystemProviderConfig {
    pub provider: String,
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(FileSystemProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::FileSystemProvider,
    config_type: FileSystemProviderConfig,
    entry_type: FileSystemProviderEntry,
    slice_name: FILE_SYSTEM_PROVIDERS,
    resolve_fn: resolve_file_system_provider,
    list_fn: list_file_system_providers
);
