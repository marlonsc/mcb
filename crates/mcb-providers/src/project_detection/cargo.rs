//! Cargo/Rust project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use cargo_toml::Manifest;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};

use super::PROJECT_DETECTORS;

/// Cargo project detector
pub struct CargoDetector;

impl CargoDetector {
    /// Create a new Cargo detector
    #[must_use]
    pub fn new(_config: &ProjectDetectorConfig) -> Self {
        Self
    }
}

#[async_trait]
impl ProjectDetector for CargoDetector {
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let manifest_path = path.join("Cargo.toml");
        if !manifest_path.exists() {
            return Ok(None);
        }

        let manifest = match Manifest::from_path(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!(path = ?manifest_path, error = %e, "Failed to parse Cargo.toml");
                return Ok(None);
            }
        };

        let Some(package) = manifest.package else {
            // Workspace root without package
            tracing::debug!(path = ?manifest_path, "Cargo.toml is workspace root, no package");
            return Ok(None);
        };

        let name = package.name.clone();
        let version = package
            .version
            .get()
            .map(ToString::to_string)
            .unwrap_or_default();

        let dependencies: Vec<String> = manifest.dependencies.keys().cloned().collect();

        Ok(Some(ProjectType::Cargo {
            name,
            version,
            dependencies,
        }))
    }

    fn marker_files(&self) -> &[&str] {
        &["Cargo.toml"]
    }

    fn detector_name(&self) -> &str {
        "cargo"
    }
}

fn cargo_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    Ok(Arc::new(CargoDetector::new(config)))
}

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(PROJECT_DETECTORS)]
static CARGO_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "cargo",
    description: "Detects Rust projects with Cargo.toml",
    marker_files: &["Cargo.toml"],
    factory: cargo_factory,
};
