//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::{Path, PathBuf};

use crate::config::NamingRulesConfig;
use crate::constants::common::MCB_CRATE_PREFIX;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::run_context::ValidationRunContext;
use crate::{Result, ValidationConfig};

use super::checks::{
    validate_ca_naming, validate_constant_names, validate_file_suffix, validate_function_names,
    validate_module_name, validate_type_names,
};
use super::violation::NamingViolation;

/// Validates naming conventions across Rust code.
///
/// Checks that structs, enums, traits use CamelCase; functions and methods use `snake_case`;
/// constants use `SCREAMING_SNAKE_CASE`; and modules/files use `snake_case`.
pub struct NamingValidator {
    config: ValidationConfig,
    rules: NamingRulesConfig,
}

crate::impl_rules_validator_new!(NamingValidator, naming);

impl NamingValidator {
    /// Creates a validator with custom configuration.
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &NamingRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Runs all naming validations and returns collected violations.
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation, directory enumeration, or file reading fails.
    pub fn validate_all(&self) -> Result<Vec<NamingViolation>> {
        if !self.rules.enabled {
            mcb_domain::debug!("naming", "Naming validator disabled, skipping");
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();

        let t = std::time::Instant::now();
        let v = self.run_type_name_check()?;
        mcb_domain::debug!(
            "naming",
            "type_names done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = self.run_function_name_check()?;
        mcb_domain::debug!(
            "naming",
            "function_names done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = self.run_constant_name_check()?;
        mcb_domain::debug!(
            "naming",
            "constant_names done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = self.run_module_name_check()?;
        mcb_domain::debug!(
            "naming",
            "module_names done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = self.run_file_suffix_check()?;
        mcb_domain::debug!(
            "naming",
            "file_suffix done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        let t = std::time::Instant::now();
        let v = self.run_ca_naming_check()?;
        mcb_domain::debug!(
            "naming",
            "ca_naming done",
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);

        Ok(violations)
    }

    /// Validates that struct, enum, and trait names follow CamelCase convention.
    fn run_type_name_check(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;
            violations.extend(validate_type_names(path, &content));
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates that function and method names follow `snake_case` convention.
    fn run_function_name_check(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;
            violations.extend(validate_function_names(path, &content));
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates that constants and statics follow `SCREAMING_SNAKE_CASE` convention.
    fn run_constant_name_check(&self) -> Result<Vec<NamingViolation>> {
        let const_pattern = compile_regex(r"(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")?;
        let static_pattern = compile_regex(r"(?:pub\s+)?static\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")?;

        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;
            violations.extend(validate_constant_names(
                path,
                &content,
                &const_pattern,
                &static_pattern,
            ));
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates that module and file names follow `snake_case` convention.
    fn run_module_name_check(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            if let Some(violation) = validate_module_name(path) {
                violations.push(violation);
            }
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates that file suffixes match component types per Clean Architecture naming conventions.
    fn run_file_suffix_check(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let crate_name = self.crate_name_from_path(path);
            if let Some(violation) = validate_file_suffix(
                path,
                &crate_name,
                &self.rules.server_crate,
                &self.rules.domain_crate,
            ) {
                violations.push(violation);
            }
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates Clean Architecture naming conventions for files and components.
    fn run_ca_naming_check(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let crate_name = self.crate_name_from_path(path);
            if let Some(violation) = validate_ca_naming(
                path,
                &crate_name,
                &self.rules.domain_crate,
                &self.rules.infrastructure_crate,
                &self.rules.server_crate,
            ) {
                violations.push(violation);
            }
            Ok(())
        })?;
        Ok(violations)
    }

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }

    fn crate_name_from_path(&self, path: &Path) -> String {
        for component in path.components() {
            let Some(component) = component.as_os_str().to_str() else {
                continue;
            };
            if component.starts_with(MCB_CRATE_PREFIX) {
                return match component {
                    "mcb-domain" => self.rules.domain_crate.clone(),
                    "mcb-infrastructure" => self.rules.infrastructure_crate.clone(),
                    "mcb-server" => self.rules.server_crate.clone(),
                    _ => String::new(),
                };
            }
        }
        String::new()
    }

    fn should_skip_crate(&self, src_dir: &Path) -> bool {
        let Some(path_str) = src_dir.to_str() else {
            return false;
        };
        let file_config = crate::config::FileConfig::load(&self.config.workspace_root);
        file_config
            .general
            .skip_crates
            .iter()
            .any(|skip| path_str.contains(skip))
    }

    fn for_each_crate_src_rs_path<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let context = ValidationRunContext::active_or_build(&self.config)?;
        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() || self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in context.file_inventory() {
                if entry.absolute_path.starts_with(&src_dir)
                    && entry.detected_language == Some(LanguageId::Rust)
                {
                    f(&entry.absolute_path)?;
                }
            }
        }
        Ok(())
    }
}

crate::impl_validator!(
    NamingValidator,
    "naming",
    "Validates naming conventions (CamelCase, snake_case, SCREAMING_SNAKE_CASE)"
);
