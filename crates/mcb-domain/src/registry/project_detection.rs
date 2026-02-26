use std::collections::HashMap;

/// Configuration for project detection service resolution.
#[derive(Debug, Clone, Default)]
pub struct ProjectDetectionServiceConfig {
    /// Provider name (e.g. "universal").
    pub provider: String,
    /// Repository root path.
    pub repo_path: Option<String>,
    /// Additional provider-specific configuration.
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(ProjectDetectionServiceConfig {
    /// Set the repository root path.
    repo_path: with_repo_path(into String),
});

crate::impl_registry!(
    provider_trait: crate::ports::ProjectDetectorService,
    config_type: ProjectDetectionServiceConfig,
    entry_type: ProjectDetectionServiceEntry,
    slice_name: PROJECT_DETECTION_SERVICES,
    resolve_fn: resolve_project_detection_service,
    list_fn: list_project_detection_services
);
