//! Naming Convention Validation
//!
//! Validates naming conventions:
//! - Structs/Enums/Traits: CamelCase
//! - Functions/Methods: `snake_case`
//! - Constants: `SCREAMING_SNAKE_CASE`
//! - Modules/Files: `snake_case`

use std::path::{Path, PathBuf};

use crate::config::NamingRulesConfig;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::run_context::ValidationRunContext;
use crate::{Result, ValidationConfig};

mod checks;
pub mod constants;
mod violation;

use checks::{
    validate_ca_naming, validate_constant_names, validate_file_suffix, validate_function_names,
    validate_module_name, validate_type_names,
};
pub use violation::NamingViolation;

/// Validates naming conventions across Rust code.
///
/// Checks that structs, enums, traits use CamelCase; functions and methods use `snake_case`;
/// constants use `SCREAMING_SNAKE_CASE`; and modules/files use `snake_case`.
pub struct NamingValidator {
    config: ValidationConfig,
    rules: NamingRulesConfig,
}

impl NamingValidator {
    /// Creates a new naming validator, loading configuration from files.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.naming)
    }

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
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.run_type_name_check()?);
        violations.extend(self.run_function_name_check()?);
        violations.extend(self.run_constant_name_check()?);
        violations.extend(self.run_module_name_check()?);
        violations.extend(self.run_file_suffix_check()?);
        violations.extend(self.run_ca_naming_check()?);
        Ok(violations)
    }

    /// Validates that struct, enum, and trait names follow CamelCase convention.
    fn run_type_name_check(&self) -> Result<Vec<NamingViolation>> {
        let struct_pattern = compile_regex(r"(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)")?;
        let enum_pattern = compile_regex(r"(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)")?;
        let trait_pattern = compile_regex(r"(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)")?;

        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;
            violations.extend(validate_type_names(
                path,
                &content,
                &struct_pattern,
                &enum_pattern,
                &trait_pattern,
            ));
            Ok(())
        })?;
        Ok(violations)
    }

    /// Validates that function and method names follow `snake_case` convention.
    fn run_function_name_check(&self) -> Result<Vec<NamingViolation>> {
        let fn_pattern =
            compile_regex(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")?;

        let mut violations = Vec::new();
        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;
            violations.extend(validate_function_names(path, &content, &fn_pattern));
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
            if component.starts_with("mcb-") {
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
