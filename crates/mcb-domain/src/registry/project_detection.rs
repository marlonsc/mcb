use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ProjectDetectionServiceConfig {
    pub provider: String,
    pub repo_path: Option<String>,
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(ProjectDetectionServiceConfig {
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
