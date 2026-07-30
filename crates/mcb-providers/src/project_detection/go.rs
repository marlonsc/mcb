//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Go project detector.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::error::Result;
use mcb_domain::ports::ProjectDetector;
use mcb_domain::registry::ProjectDetectorConfig;
use regex::Regex;

use super::common::read_file_opt;

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

    /// Parse the contents of a `go.mod` file into module name, Go version, and dependencies.
    fn parse_gomod(&self, content: &str) -> GoModInfo {
        let mut info = GoModInfo::default();
        let mut in_require_block = false;

        for raw in content.lines() {
            let line = raw.trim();

            if let Some(module) = Self::first_capture(&self.module_re, line) {
                info.module = module;
            }
            if let Some(version) = Self::first_capture(&self.go_version_re, line) {
                info.go_version = version;
            }

            if line.starts_with("require (") {
                in_require_block = true;
            } else if line == ")" {
                in_require_block = false;
            } else if let Some(dep) = self.parse_require_line(line, in_require_block) {
                info.dependencies.push(dep);
            }
        }

        info
    }

    /// Extract a dependency from a `require` line, whether inside a block or single-line.
    fn parse_require_line(&self, line: &str, in_require_block: bool) -> Option<String> {
        if in_require_block {
            return Self::first_capture(&self.require_re, line);
        }
        if line.starts_with("require ") && !line.contains('(') {
            return Self::first_capture(&self.require_re, &line["require ".len()..]);
        }
        None
    }

    fn first_capture(re: &Regex, line: &str) -> Option<String> {
        re.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_owned())
    }
}

/// Parsed contents of a `go.mod` file.
#[derive(Default)]
struct GoModInfo {
    module: String,
    go_version: String,
    dependencies: Vec<String>,
}

#[async_trait]
/// Go project detector implementation.
impl ProjectDetector for GoDetector {
    /// Detects a Go project by analyzing `go.mod`.
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>> {
        let gomod_path = path.join("go.mod");
        if !gomod_path.exists() {
            return Ok(None);
        }

        let Some(content) = read_file_opt(&gomod_path, "go").await else {
            return Ok(None);
        };

        let parsed = self.parse_gomod(&content);
        if parsed.module.is_empty() {
            return Ok(None);
        }

        Ok(Some(ProjectType::Go {
            module: parsed.module,
            go_version: parsed.go_version,
            dependencies: parsed.dependencies,
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

mcb_domain::register_project_detector!(
    "go",
    "Detects Go projects with go.mod",
    &["go.mod"],
    |config| {
        GoDetector::new(config)
            .map(|detector| std::sync::Arc::new(detector) as std::sync::Arc<dyn ProjectDetector>)
            .map_err(|e| {
                mcb_domain::Error::configuration(format!("Failed to initialize Go detector: {e}"))
            })
    }
);
