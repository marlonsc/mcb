use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct VcsProviderConfig {
    pub provider: String,
    pub root_path: Option<String>,
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(VcsProviderConfig {
    root_path: with_root_path(into String),
});

crate::impl_registry!(
    provider_trait: crate::ports::VcsProvider,
    config_type: VcsProviderConfig,
    entry_type: VcsProviderEntry,
    slice_name: VCS_PROVIDERS,
    resolve_fn: resolve_vcs_provider,
    list_fn: list_vcs_providers
);
