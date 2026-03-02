//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! High-level project detection facade.

use std::path::Path;

use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::ProjectDetectorConfig;

use mcb_domain::ports::PROJECT_DETECTORS;

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
                    mcb_domain::debug!(
                        "detector",
                        "Project detected",
                        &format!(
                            "detector = {}, project = {:?}",
                            entry.name,
                            project_type.name()
                        )
                    );
                    results.push(project_type);
                }
                Ok(None) => {}
                Err(e) => {
                    mcb_domain::warn!(
                        "detector",
                        "Project detection failed",
                        &format!("detector = {}, error = {}", entry.name, e)
                    );
                }
            },
            Err(e) => {
                mcb_domain::warn!(
                    "detector",
                    "Failed to create detector",
                    &format!("detector = {}, error = {}", entry.name, e)
                );
            }
        }
    }

    results
}
