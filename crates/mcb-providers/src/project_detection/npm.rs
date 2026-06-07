//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! npm/Node.js project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};
use serde::Deserialize;

use super::common::{parse_json_opt, read_file_opt};

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
    /// Create a new NPM detector
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

        let Some(content) = read_file_opt(&package_path, "npm").await else {
            return Ok(None);
        };

        let Some(package) = parse_json_opt::<PackageJson>(&content, &package_path, "npm") else {
            return Ok(None);
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

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(mcb_domain::ports::PROJECT_DETECTORS)]
static NPM_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "npm",
    description: "Detects Node.js projects with package.json",
    marker_files: &["package.json"],
    build: npm_factory,
};
