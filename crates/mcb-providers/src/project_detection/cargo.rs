//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Cargo/Rust project detector.

use std::path::Path;

use async_trait::async_trait;
use cargo_toml::Manifest;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::ProjectDetector;

use super::common::{parse_toml_opt, read_file_opt};

/// Cargo project detector
pub struct CargoDetector;

impl CargoDetector {
    /// Create a new Cargo detector
    #[must_use]
    pub fn new(_config: &mcb_domain::registry::project_detector::ProjectDetectorConfig) -> Self {
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

        let Some(content) = read_file_opt(&manifest_path, "cargo").await else {
            return Ok(None);
        };

        let Some(manifest) = parse_toml_opt::<Manifest>(&content, &manifest_path, "cargo") else {
            return Ok(None);
        };

        let Some(package) = manifest.package else {
            // Workspace root without package
            mcb_domain::debug!(
                "cargo",
                "Cargo.toml is workspace root, no package",
                &format!("path = {manifest_path:?}")
            );
            return Ok(None);
        };

        let name = package.name.clone();
        let version = package
            .version
            .get()
            .map(ToOwned::to_owned)
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

mcb_domain::register_project_detector!(
    "cargo",
    "Detects Rust projects with Cargo.toml",
    &["Cargo.toml"],
    |config| Ok(std::sync::Arc::new(CargoDetector::new(config)))
);
