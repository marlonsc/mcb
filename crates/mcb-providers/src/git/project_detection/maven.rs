//! Maven project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::project_detection::{
    ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use tokio::fs::read_to_string;

use super::PROJECT_DETECTORS;

/// Maven project detector
pub struct MavenDetector;

impl MavenDetector {
    /// Create a new Maven detector
    #[must_use]
    pub fn new(_config: &ProjectDetectorConfig) -> Self {
        Self
    }

    fn parse_pom(content: &str) -> Option<(String, String, String, Vec<String>)> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut group_id = String::new();
        let mut artifact_id = String::new();
        let mut version = String::new();
        let mut dependencies = Vec::new();

        let mut current_path: Vec<String> = Vec::new();
        let mut in_dependency = false;
        let mut dep_group_id = String::new();
        let mut dep_artifact_id = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_path.push(name.clone());

                    if Self::path_matches(&current_path, &["project", "dependencies", "dependency"])
                    {
                        in_dependency = true;
                        dep_group_id.clear();
                        dep_artifact_id.clear();
                    }
                }
                Ok(Event::Text(e)) => {
                    // In quick-xml 0.39, BytesText doesn't have unescape() method
                    // We decode the bytes directly to string
                    let text = String::from_utf8_lossy(e.as_ref()).to_string();

                    if Self::path_matches(&current_path, &["project", "groupId"])
                        && group_id.is_empty()
                    {
                        group_id = text;
                    } else if Self::path_matches(&current_path, &["project", "artifactId"])
                        && artifact_id.is_empty()
                    {
                        artifact_id = text;
                    } else if Self::path_matches(&current_path, &["project", "version"])
                        && version.is_empty()
                    {
                        version = text;
                    } else if Self::path_matches(
                        &current_path,
                        &["project", "dependencies", "dependency", "groupId"],
                    ) && in_dependency
                    {
                        dep_group_id = text;
                    } else if Self::path_matches(
                        &current_path,
                        &["project", "dependencies", "dependency", "artifactId"],
                    ) && in_dependency
                    {
                        dep_artifact_id = text;
                    }
                }
                Ok(Event::End(_)) => {
                    if Self::path_matches(&current_path, &["project", "dependencies", "dependency"])
                        && in_dependency
                    {
                        if !dep_artifact_id.is_empty() {
                            let dep = if dep_group_id.is_empty() {
                                std::mem::take(&mut dep_artifact_id)
                            } else {
                                format!("{}:{}", dep_group_id, dep_artifact_id)
                            };
                            dependencies.push(dep);
                        }
                        in_dependency = false;
                    }
                    current_path.pop();
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                Ok(_) => continue,
            }
        }

        if artifact_id.is_empty() {
            return None;
        }

        Some((group_id, artifact_id, version, dependencies))
    }

    fn path_matches(current: &[String], expected: &[&str]) -> bool {
        if current.len() != expected.len() {
            return false;
        }
        current
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| a.as_str() == *b)
    }
}

#[async_trait]
impl ProjectDetector for MavenDetector {
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let pom_path = path.join("pom.xml");
        if !pom_path.exists() {
            return Ok(None);
        }

        let content = match read_to_string(&pom_path).await {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(path = ?pom_path, error = %e, "Failed to read pom.xml");
                return Ok(None);
            }
        };

        match Self::parse_pom(&content) {
            Some((group_id, artifact_id, version, dependencies)) => Ok(Some(ProjectType::Maven {
                group_id,
                artifact_id,
                version,
                dependencies,
            })),
            None => Ok(None),
        }
    }

    fn marker_files(&self) -> &[&str] {
        &["pom.xml"]
    }

    fn detector_name(&self) -> &str {
        "maven"
    }
}

fn maven_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    Ok(Arc::new(MavenDetector::new(config)))
}

#[linkme::distributed_slice(PROJECT_DETECTORS)]
static MAVEN_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "maven",
    description: "Detects Maven projects with pom.xml",
    marker_files: &["pom.xml"],
    factory: maven_factory,
};
