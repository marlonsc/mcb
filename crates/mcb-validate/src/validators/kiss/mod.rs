use std::path::PathBuf;

use crate::config::KISSRulesConfig;
use crate::thresholds::thresholds;
use crate::{Result, ValidationConfig};

mod checks;
mod helpers;
mod violations;

pub use self::violations::KissViolation;

pub struct KissValidator {
    config: ValidationConfig,
    rules: KISSRulesConfig,
    max_struct_fields: usize,
    max_function_params: usize,
    max_builder_fields: usize,
    max_nesting_depth: usize,
    max_function_lines: usize,
}

impl KissValidator {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.kiss)
    }

    pub fn with_config(config: ValidationConfig, rules: &KISSRulesConfig) -> Self {
        let t = thresholds();
        Self {
            config,
            rules: rules.clone(),
            max_struct_fields: t.max_struct_fields,
            max_function_params: t.max_function_params,
            max_builder_fields: t.max_builder_fields,
            max_nesting_depth: t.max_nesting_depth,
            max_function_lines: t.max_function_lines,
        }
    }

    #[must_use]
    pub fn with_max_struct_fields(mut self, max: usize) -> Self {
        self.max_struct_fields = max;
        self
    }

    #[must_use]
    pub fn with_max_function_params(mut self, max: usize) -> Self {
        self.max_function_params = max;
        self
    }

    pub fn validate_all(&self) -> Result<Vec<KissViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_struct_fields()?);
        violations.extend(self.validate_function_params()?);
        violations.extend(self.validate_builder_complexity()?);
        violations.extend(self.validate_nesting_depth()?);
        violations.extend(self.validate_function_length()?);
        Ok(violations)
    }
}

crate::impl_validator!(
    KissValidator,
    "kiss",
    "Validates KISS principle (Keep It Simple, Stupid)"
);
