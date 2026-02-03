//! npm/Node.js project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs::read_to_string;

use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::project_detection::{
    ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry,
};

use super::PROJECT_DETECTORS;

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<std::collections::HashMap<String, String>>,
}

/// npm project detector
pub struct NpmDetector;

impl NpmDetector {
    #[must_use]
    pub fn new(_config: &ProjectDetectorConfig) -> Self {
        Self
    }
}

#[async_trait]
impl ProjectDetector for NpmDetector {
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let package_path = path.join("package.json");
        if !package_path.exists() {
            return Ok(None);
        }

        let content = match read_to_string(&package_path).await {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(path = ?package_path, error = %e, "Failed to read package.json");
                return Ok(None);
            }
        };

        let package: PackageJson = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!(path = ?package_path, error = %e, "Failed to parse package.json");
                return Ok(None);
            }
        };

        let name = package.name.unwrap_or_default();
        let version = package.version.unwrap_or_default();

        let mut dependencies: Vec<String> = package
            .dependencies
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect();

        // Include dev dependencies
        if let Some(dev_deps) = package.dev_dependencies {
            dependencies.extend(dev_deps.keys().cloned());
        }

        dependencies.sort();
        dependencies.dedup();

        Ok(Some(ProjectType::Npm {
            name,
            version,
            dependencies,
        }))
    }

    fn marker_files(&self) -> &[&str] {
        &["package.json"]
    }

    fn detector_name(&self) -> &str {
        "npm"
    }
}

fn npm_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    Ok(Arc::new(NpmDetector::new(config)))
}

#[linkme::distributed_slice(PROJECT_DETECTORS)]
static NPM_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "npm",
    description: "Detects Node.js projects with package.json",
    marker_files: &["package.json"],
    factory: npm_factory,
};
