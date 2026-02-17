//! Python project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};
use serde::Deserialize;
use tokio::fs::read_to_string;

use super::PROJECT_DETECTORS;

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
            && let Ok(content) = read_to_string(&pyproject_path).await
            && let Ok(pyproject) = toml::from_str::<PyProject>(&content)
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
            && let Ok(content) = read_to_string(&requirements_path).await
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

fn python_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    Ok(Arc::new(PythonDetector::new(config)))
}

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(PROJECT_DETECTORS)]
static PYTHON_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "python",
    description: "Detects Python projects with pyproject.toml or requirements.txt",
    marker_files: &["pyproject.toml", "requirements.txt"],
    build: python_factory,
};
