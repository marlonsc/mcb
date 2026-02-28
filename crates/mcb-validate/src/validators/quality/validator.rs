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
        violations.extend(unwrap::validate(self)?);
        violations.extend(panic::validate(self)?);
        violations.extend(metrics::validate(self)?);
        violations.extend(comments::validate(self)?);
        violations.extend(dead_code::validate(self)?);
        Ok(violations)
    }
}

impl crate::traits::validator::Validator for QualityValidator {
    fn name(&self) -> &'static str {
        "quality"
    }

    fn description(&self) -> &'static str {
        "Validates code quality (no unwrap/expect)"
    }

    fn checks<'a>(
        &'a self,
        _config: &'a crate::ValidationConfig,
    ) -> crate::Result<Vec<crate::traits::validator::NamedCheck<'a>>> {
        Ok(vec![
            crate::traits::validator::NamedCheck::new("unwrap", move || {
                Ok(unwrap::validate(self)?
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
                    .collect())
            }),
            crate::traits::validator::NamedCheck::new("panic", move || {
                Ok(panic::validate(self)?
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
                    .collect())
            }),
            crate::traits::validator::NamedCheck::new("metrics", move || {
                Ok(metrics::validate(self)?
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
                    .collect())
            }),
            crate::traits::validator::NamedCheck::new("comments", move || {
                Ok(comments::validate(self)?
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
                    .collect())
            }),
            crate::traits::validator::NamedCheck::new("dead_code", move || {
                Ok(dead_code::validate(self)?
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
                    .collect())
            }),
        ])
    }
}
