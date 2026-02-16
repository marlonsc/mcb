use std::path::{Path, PathBuf};

use super::catch_all::validate_empty_catch_alls;
use super::empty::validate_empty_methods;
use super::hardcoded::validate_hardcoded_returns;
use super::logging::validate_log_only_methods;
use super::stubs::validate_stub_macros;
use super::wrappers::validate_pass_through_wrappers;
use crate::constants::common::TEST_DIR_FRAGMENT;
use crate::pattern_registry::required_pattern;

use crate::config::ImplementationRulesConfig;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, ValidationConfig};

use super::super::violation::ImplementationViolation;

/// Implementation quality validator
pub struct ImplementationQualityValidator {
    config: ValidationConfig,
    rules: ImplementationRulesConfig,
}

impl ImplementationQualityValidator {
    /// Create a new implementation quality validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let config = ValidationConfig::new(workspace_root);
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        Self { config, rules }
    }

    /// Create a validator with custom configuration
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &ImplementationRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Run all implementation quality validations
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or pattern compilation fails.
    pub fn validate_all(&self) -> Result<Vec<ImplementationViolation>> {
        let mut files = Vec::new();
        for_each_scan_file(
            &self.config,
            Some(LanguageId::Rust),
            false,
            |entry, src_dir| {
                if self.should_skip_crate(src_dir) || is_test_path(&entry.absolute_path) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(&entry.absolute_path)?;
                files.push((entry.absolute_path.clone(), content));
                Ok(())
            },
        )?;

        let fn_pattern = required_pattern("IMPL001.fn_decl")?;

        let mut all = Vec::new();
        all.extend(validate_empty_methods(&files, fn_pattern)?);
        all.extend(validate_hardcoded_returns(&files, fn_pattern)?);
        all.extend(validate_stub_macros(&files, fn_pattern)?);
        all.extend(validate_empty_catch_alls(&files)?);
        all.extend(validate_pass_through_wrappers(&files, fn_pattern)?);
        all.extend(validate_log_only_methods(&files, fn_pattern)?);
        Ok(all)
    }

    /// Check if a crate should be skipped based on configuration
    fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let Some(path_str) = src_dir.to_str() else {
            return false;
        };
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}

fn is_test_path(path: &Path) -> bool {
    path.to_str()
        .is_some_and(|path| path.contains(TEST_DIR_FRAGMENT))
}

crate::impl_validator!(
    ImplementationQualityValidator,
    "implementation",
    "Validates implementation quality patterns (empty methods, hardcoded returns, stubs)"
);
