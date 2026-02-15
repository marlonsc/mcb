//! Pattern Compliance Validation
//!
//! This module provides the `PatternValidator` which ensures code patterns across the
//! workspace follow established best practices and architectural constraints.
//! It validates Dependency Injection (DI) usage, async trait implementation details,
//! and consistency in Result/Error types.
//!
//! # Code Smells
//!
//! Consider splitting into separate modules for DI, async traits, and result types.
//!
//! Validates code patterns:
//! - DI uses `Arc<dyn Trait>` not `Arc<ConcreteType>`
//! - Async traits have `#[async_trait]` and Send + Sync bounds
//! - Error types use `crate::error::Result<T>`
//! - Provider pattern compliance

use std::path::PathBuf;

use crate::config::PatternRulesConfig;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, ValidationConfig};

mod async_check;
mod di;
mod result_check;
mod violation;

pub use violation::PatternViolation;

/// Pattern validator
pub struct PatternValidator {
    config: ValidationConfig,
    rules: PatternRulesConfig,
}

impl PatternValidator {
    /// Create a new pattern validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.patterns)
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig, rules: &PatternRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Run all pattern validations
    pub fn validate_all(&self) -> Result<Vec<PatternViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_trait_based_di()?);
        violations.extend(self.validate_async_traits()?);
        violations.extend(self.validate_result_types()?);
        Ok(violations)
    }

    /// Verify `Arc<dyn Trait>` pattern instead of `Arc<ConcreteType>`.
    pub fn validate_trait_based_di(&self) -> Result<Vec<PatternViolation>> {
        // Pattern to find Arc<SomeConcreteType> where SomeConcreteType doesn't start with "dyn"
        let arc_pattern = compile_regex(&self.rules.arc_pattern)
            .or_else(|_| compile_regex(r"Arc<([A-Z][a-zA-Z0-9_]*)>"))?;

        // Known concrete types that are OK to use directly
        let allowed_concrete = &self.rules.allowed_concrete_types;

        // Provider trait names that should use Arc<dyn ...>
        let provider_traits = &self.rules.provider_trait_suffixes;

        self.scan_rust_files(|path, content| {
            di::check_arc_usage(
                path,
                content,
                &arc_pattern,
                allowed_concrete,
                provider_traits,
            )
        })
    }

    /// Check async traits have #[`async_trait`] and Send + Sync bounds.
    pub fn validate_async_traits(&self) -> Result<Vec<PatternViolation>> {
        self.scan_rust_files(async_check::check_async_traits)
    }

    /// Verify consistent error type usage.
    pub fn validate_result_types(&self) -> Result<Vec<PatternViolation>> {
        self.scan_rust_files_with_filter(
            |src_dir| self.should_skip_result_check(src_dir),
            result_check::check_result_types,
        )
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

    /// Check if a crate should be skipped for result checking
    fn should_skip_result_check(&self, src_dir: &std::path::Path) -> bool {
        let Some(path_str) = src_dir.to_str() else {
            return false;
        };
        self.rules
            .result_check_excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }

    fn scan_rust_files<F>(&self, visitor: F) -> Result<Vec<PatternViolation>>
    where
        F: FnMut(&std::path::Path, &str) -> Vec<PatternViolation>,
    {
        self.scan_rust_files_with_filter(|_| false, visitor)
    }

    fn scan_rust_files_with_filter<F>(
        &self,
        extra_skip_check: impl Fn(&std::path::Path) -> bool,
        mut visitor: F,
    ) -> Result<Vec<PatternViolation>>
    where
        F: FnMut(&std::path::Path, &str) -> Vec<PatternViolation>,
    {
        let mut violations = Vec::new();
        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) || extra_skip_check(&src_dir) {
                continue;
            }

            for_each_scan_file(
                &self.config,
                Some(LanguageId::Rust),
                false,
                |entry, candidate_src_dir| {
                    if candidate_src_dir != src_dir {
                        return Ok(());
                    }

                    let path = &entry.absolute_path;
                    let content = std::fs::read_to_string(path)?;
                    violations.extend(visitor(path, &content));
                    Ok(())
                },
            )?;
        }
        Ok(violations)
    }
}

crate::impl_validator!(
    PatternValidator,
    "patterns",
    "Validates code patterns (DI, async traits, error handling)"
);
