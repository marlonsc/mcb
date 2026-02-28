//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
use std::path::PathBuf;

use super::violations::QualityViolation;
use super::{comments, dead_code, metrics, panic, unwrap};
use crate::thresholds::thresholds;
use crate::{Result, ValidationConfig};

/// Validator for code quality metrics and safety checks
pub struct QualityValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) max_file_lines: usize,
    pub(crate) excluded_paths: Vec<String>,
}

impl QualityValidator {
    /// Creates a new instance of the quality validator for the given workspace.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a new validator instance using a provided configuration.
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        // Load file configuration to get quality rules
        let file_config = crate::config::FileConfig::load(&config.workspace_root);
        Self {
            config,
            max_file_lines: thresholds().max_file_lines,
            excluded_paths: file_config.rules.quality.excluded_paths,
        }
    }

    /// Configures the maximum allowed lines per file.
    #[must_use]
    pub fn with_max_file_lines(mut self, max: usize) -> Self {
        self.max_file_lines = max;
        self
    }

    /// Executes all configured quality checks and returns any violations found.
    ///
    /// # Errors
    ///
    /// Returns an error if any sub-validation encounters a file system or parsing error.
    pub fn validate_all(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();

        let t = std::time::Instant::now();
        let v = unwrap::validate(self)?;
        mcb_domain::debug!(
            "quality",
            "unwrap check done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = panic::validate(self)?;
        mcb_domain::debug!(
            "quality",
            "panic check done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = metrics::validate(self)?;
        mcb_domain::debug!(
            "quality",
            "metrics check done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = comments::validate(self)?;
        mcb_domain::debug!(
            "quality",
            "comments check done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = dead_code::validate(self)?;
        mcb_domain::debug!(
            "quality",
            "dead_code check done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        Ok(violations)
    }
}

crate::impl_validator!(
    QualityValidator,
    "quality",
    "Validates code quality (no unwrap/expect)"
);
