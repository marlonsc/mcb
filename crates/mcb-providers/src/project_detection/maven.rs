//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Maven project detector.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};
use quick_xml::Reader;
use quick_xml::events::Event;

use super::common::read_file_opt;

/// Maven project detector
pub struct MavenDetector;

struct MavenPomState {
    group_id: String,
    artifact_id: String,
    version: String,
    dependencies: Vec<String>,
    in_dependency: bool,
    dep_group_id: String,
    dep_artifact_id: String,
}

impl MavenPomState {
    fn new() -> Self {
        Self {
            group_id: String::new(),
            artifact_id: String::new(),
            version: String::new(),
            dependencies: Vec::new(),
            in_dependency: false,
            dep_group_id: String::new(),
            dep_artifact_id: String::new(),
        }
    }
}

impl MavenDetector {
    /// Create a new Maven detector
    #[must_use]
    pub fn new(_config: &ProjectDetectorConfig) -> Self {
        Self
    }

    /// Parses a `pom.xml` file to extract project metadata.
    fn parse_pom(content: &str) -> Option<(String, String, String, Vec<String>)> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut state = MavenPomState::new();

        let mut current_path: Vec<String> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_path.push(name.clone());

                    if Self::path_matches(&current_path, &["project", "dependencies", "dependency"])
                    {
                        state.in_dependency = true;
                        state.dep_group_id.clear();
                        state.dep_artifact_id.clear();
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = String::from_utf8_lossy(e.as_ref()).to_string();
                    Self::handle_text_event(&current_path, &text, &mut state);
                }
                Ok(Event::End(_)) => {
                    if Self::path_matches(&current_path, &["project", "dependencies", "dependency"])
                        && state.in_dependency
                    {
                        if !state.dep_artifact_id.is_empty() {
                            state.dependencies.push(Self::format_dependency(
                                &state.dep_group_id,
                                &state.dep_artifact_id,
                            ));
                        }
                        state.in_dependency = false;
                    }
                    current_path.pop();
                }
                Ok(Event::Eof) | Err(_) => break,
                Ok(_) => continue,
            }
        }

        if state.artifact_id.is_empty() {
            return None;
        }

        Some((
            state.group_id,
            state.artifact_id,
            state.version,
            state.dependencies,
        ))
    }

    fn handle_text_event(path: &[String], text: &str, state: &mut MavenPomState) {
        if Self::path_matches(path, &["project", "groupId"]) && state.group_id.is_empty() {
            state.group_id = text.to_owned();
        } else if Self::path_matches(path, &["project", "artifactId"])
            && state.artifact_id.is_empty()
        {
            state.artifact_id = text.to_owned();
        } else if Self::path_matches(path, &["project", "version"]) && state.version.is_empty() {
            state.version = text.to_owned();
        } else if state.in_dependency {
            if Self::path_matches(path, &["project", "dependencies", "dependency", "groupId"]) {
                state.dep_group_id = text.to_owned();
            } else if Self::path_matches(
                path,
                &["project", "dependencies", "dependency", "artifactId"],
            ) {
                state.dep_artifact_id = text.to_owned();
            }
        }
    }

    fn format_dependency(group_id: &str, artifact_id: &str) -> String {
        if group_id.is_empty() {
            artifact_id.to_owned()
        } else {
            format!("{group_id}:{artifact_id}")
        }
    }

    /// Checks if the current XML path matches the expected path.
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
/// Maven project detector implementation.
impl ProjectDetector for MavenDetector {
    /// Detects a Maven project by analyzing `pom.xml`.
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let pom_path = path.join("pom.xml");
        if !pom_path.exists() {
            return Ok(None);
        }

        let Some(content) = read_file_opt(&pom_path, "maven").await else {
            return Ok(None);
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

    /// Returns the list of files that identify a Maven project.
    fn marker_files(&self) -> &[&str] {
        &["pom.xml"]
    }

    /// Returns the detector name ("maven").
    fn detector_name(&self) -> &str {
        "maven"
    }
}

/// Factory function for creating Maven detector instances.
fn maven_factory(
    config: &ProjectDetectorConfig,
) -> mcb_domain::error::Result<Arc<dyn ProjectDetector>> {
    Ok(Arc::new(MavenDetector::new(config)))
}

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(mcb_domain::ports::PROJECT_DETECTORS)]
static MAVEN_DETECTOR: ProjectDetectorEntry = ProjectDetectorEntry {
    name: "maven",
    description: "Detects Maven projects with pom.xml",
    marker_files: &["pom.xml"],
    build: maven_factory,
};
