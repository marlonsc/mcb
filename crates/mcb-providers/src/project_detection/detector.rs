//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! High-level project detection facade.

use std::path::Path;

use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::ProjectDetectorConfig;

use super::registry::PROJECT_DETECTORS;

/// Detect all project types at a given path
/// Returns multiple `ProjectTypes` if overlapping (e.g., Cargo.toml + package.json)
pub async fn detect_all_projects(path: &Path) -> Vec<ProjectType> {
    let config = ProjectDetectorConfig {
        repo_path: path.to_str().unwrap_or_default().to_owned(),
    };

    let mut results = Vec::new();

    for entry in PROJECT_DETECTORS {
        let has_marker = entry.marker_files.iter().any(|f| path.join(f).exists());
        if !has_marker {
            continue;
        }

        match (entry.build)(&config) {
            Ok(detector) => match detector.detect(path).await {
                Ok(Some(project_type)) => {
                    tracing::debug!(
                        detector = entry.name,
                        project = ?project_type.name(),
                        "Project detected"
                    );
                    results.push(project_type);
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        detector = entry.name,
                        error = %e,
                        "Project detection failed"
                    );
                }
            },
            Err(e) => {
                tracing::warn!(
                    detector = entry.name,
                    error = %e,
                    "Failed to create detector"
                );
            }
        }
    }

    results
}
