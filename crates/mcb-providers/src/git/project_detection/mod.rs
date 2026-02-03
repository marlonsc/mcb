//! Project detection with auto-registration via linkme.

mod cargo;
mod go;
mod maven;
mod npm;
mod python;

use std::path::Path;

use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::providers::project_detection::{
    ProjectDetectorConfig, ProjectDetectorEntry,
};
use mcb_domain::utils::project_type::project_name;

// Re-export detectors for direct use
pub use cargo::CargoDetector;
pub use go::GoDetector;
pub use maven::MavenDetector;
pub use npm::NpmDetector;
pub use python::PythonDetector;

/// Distributed slice for auto-registration of project detectors
#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];

/// Detect all project types at a given path
/// Returns multiple ProjectTypes if overlapping (e.g., Cargo.toml + package.json)
pub async fn detect_all_projects(path: &Path) -> Vec<ProjectType> {
    let config = ProjectDetectorConfig {
        repo_path: path.to_string_lossy().to_string(),
    };

    let mut results = Vec::new();

    for entry in PROJECT_DETECTORS {
        // Quick check: do any marker files exist?
        let has_marker = entry.marker_files.iter().any(|f| path.join(f).exists());
        if !has_marker {
            continue;
        }

        match (entry.factory)(&config) {
            Ok(detector) => match detector.detect(path).await {
                Ok(Some(project_type)) => {
                    tracing::debug!(
                        detector = entry.name,
                        project = ?project_name(&project_type),
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
