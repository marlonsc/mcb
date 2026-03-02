use std::collections::HashMap;

/// Configuration for VCS provider resolution.
#[derive(Debug, Clone, Default)]
pub struct VcsProviderConfig {
    /// Provider name (e.g. "git").
    pub provider: String,
    /// Repository root path.
    pub root_path: Option<String>,
    /// Additional provider-specific configuration.
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(VcsProviderConfig {
    /// Set the repository root path.
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
