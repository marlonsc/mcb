//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Go project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};
use regex::Regex;
use tokio::fs::read_to_string;

use super::PROJECT_DETECTORS;

/// Go project detector
pub struct GoDetector {
    module_re: Regex,
    go_version_re: Regex,
    require_re: Regex,
}

impl GoDetector {
    /// Create a new Go detector
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation fails.
    pub fn new(_config: &ProjectDetectorConfig) -> std::result::Result<Self, regex::Error> {
        let module_re = Regex::new(r"^module\s+(\S+)")?;
        let go_version_re = Regex::new(r"^go\s+(\d+\.\d+)")?;
        let require_re = Regex::new(r"^\s*(\S+)\s+v")?;

        Ok(Self {
            module_re,
            go_version_re,
            require_re,
        })
    }
}

#[async_trait]
/// Go project detector implementation.
impl ProjectDetector for GoDetector {
    /// Detects a Go project by analyzing `go.mod`.
    // TODO(qlty): Function with high complexity (count = 19): detect
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let gomod_path = path.join("go.mod");
        if !gomod_path.exists() {
            return Ok(None);
        }

        let content = match read_to_string(&gomod_path).await {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(path = ?gomod_path, error = %e, "Failed to read go.mod");
                return Ok(None);
            }
        };

        let mut module = String::new();
        let mut go_version = String::new();
        let mut dependencies = Vec::new();
        let mut in_require_block = false;

        for line in content.lines() {
            let line = line.trim();

            if let Some(caps) = self.module_re.captures(line) {
                module = caps
                    .get(1)
                    .map(|m| m.as_str().to_owned())
                    .unwrap_or_default();
            }

            if let Some(caps) = self.go_version_re.captures(line) {
                go_version = caps
                    .get(1)
                    .map(|m| m.as_str().to_owned())
                    .unwrap_or_default();
            }

            if line.starts_with("require (") {
                in_require_block = true;
                continue;
            }

            if line == ")" {
                in_require_block = false;
                continue;
            }

            if in_require_block
                && let Some(caps) = self.require_re.captures(line)
                && let Some(dep) = caps.get(1)
            {
                dependencies.push(dep.as_str().to_owned());
            }

            // Single-line require
            if line.starts_with("require ")
                && !line.contains('(')
                && let Some(caps) = self.require_re.captures(&line["require ".len()..])
                && let Some(dep) = caps.get(1)
            {
                dependencies.push(dep.as_str().to_owned());
            }
        }

        if module.is_empty() {
            return Ok(None);
        }

        Ok(Some(ProjectType::Go {
            module,
            go_version,
            dependencies,
        }))
    }

    /// Returns the list of files that identify a Go project.
    fn marker_files(&self) -> &[&str] {
        &["go.mod"]
    }

    /// Returns the detector name ("go").
    fn detector_name(&self) -> &str {
        "go"
    }
}

/// Factory function for creating Go detector instances.
fn go_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    GoDetector::new(config)
        .map(|detector| Arc::new(detector) as Arc<dyn ProjectDetector>)
        .map_err(|e| {
            mcb_domain::Error::configuration(format!("Failed to initialize Go detector: {e}"))
        })
}

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(PROJECT_DETECTORS)]
static GO_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "go",
    description: "Detects Go projects with go.mod",
    marker_files: &["go.mod"],
    build: go_factory,
};
