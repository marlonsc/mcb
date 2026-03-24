//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Python project detector.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::ProjectDetector;
use mcb_domain::registry::ProjectDetectorConfig;
use serde::Deserialize;

use super::common::{parse_toml_opt, read_file_opt};

#[derive(Deserialize)]
struct PyProject {
    project: Option<PyProjectProject>,
}

#[derive(Deserialize)]
struct PyProjectProject {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<Vec<String>>,
}

/// Python project detector
pub struct PythonDetector;

impl PythonDetector {
    /// Create a new Python detector
    #[must_use]
    pub fn new(_config: &ProjectDetectorConfig) -> Self {
        Self
    }

    fn parse_requirements(content: &str) -> Vec<String> {
        content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                // Extract package name before version specifier
                line.split(['=', '<', '>', '[', ';'])
                    .next()
                    .unwrap_or(line)
                    .trim()
                    .to_owned()
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[async_trait]
impl ProjectDetector for PythonDetector {
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let pyproject_path = path.join("pyproject.toml");
        let requirements_path = path.join("requirements.txt");

        // Try pyproject.toml first (modern standard)
        if pyproject_path.exists()
            && let Some(content) = read_file_opt(&pyproject_path, "python").await
            && let Some(pyproject) =
                parse_toml_opt::<PyProject>(&content, &pyproject_path, "python")
            && let Some(project) = pyproject.project
        {
            return Ok(Some(ProjectType::Python {
                name: project.name.unwrap_or_else(|| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_owned()
                }),
                version: project.version,
                dependencies: project.dependencies.unwrap_or_default(),
            }));
        }

        // Fall back to requirements.txt
        if requirements_path.exists()
            && let Some(content) = read_file_opt(&requirements_path, "python").await
        {
            let dependencies = Self::parse_requirements(&content);
            return Ok(Some(ProjectType::Python {
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_owned(),
                version: None,
                dependencies,
            }));
        }

        Ok(None)
    }

    fn marker_files(&self) -> &[&str] {
        &["pyproject.toml", "requirements.txt"]
    }

    fn detector_name(&self) -> &str {
        "python"
    }
}

mcb_domain::register_project_detector!(
    "python",
    "Detects Python projects with pyproject.toml or requirements.txt",
    &["pyproject.toml", "requirements.txt"],
    |_config| Ok(std::sync::Arc::new(PythonDetector))
);
