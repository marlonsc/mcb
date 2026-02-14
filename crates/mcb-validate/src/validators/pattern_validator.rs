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

use crate::filters::LanguageId;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::config::PatternRulesConfig;
use crate::scan::for_each_scan_file;
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

static TRAIT_PATTERN: OnceLock<Regex> = OnceLock::new();
static ASYNC_FN_PATTERN: OnceLock<Regex> = OnceLock::new();
static SEND_SYNC_PATTERN: OnceLock<Regex> = OnceLock::new();
static ASYNC_TRAIT_ATTR: OnceLock<Regex> = OnceLock::new();
static ALLOW_ASYNC_FN_TRAIT: OnceLock<Regex> = OnceLock::new();
static STD_RESULT_PATTERN: OnceLock<Regex> = OnceLock::new();
static EXPLICIT_RESULT_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Pattern violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternViolation {
    /// Concrete type used in DI instead of trait object
    ConcreteTypeInDi {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The concrete type found.
        concrete_type: String,
        /// Suggested replacement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Async trait missing Send + Sync bounds
    MissingSendSync {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// The missing bounds.
        missing_bound: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Async trait missing #[`async_trait`] attribute
    MissingAsyncTrait {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Using `std::result::Result` instead of `crate::error::Result`
    RawResultType {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Context code snippet.
        context: String,
        /// Suggested replacement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Missing Interface trait bound for dill DI
    MissingInterfaceBound {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl PatternViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for PatternViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConcreteTypeInDi {
                file,
                line,
                concrete_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Concrete type in DI: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    concrete_type,
                    suggestion
                )
            }
            Self::MissingSendSync {
                file,
                line,
                trait_name,
                missing_bound,
                ..
            } => {
                write!(
                    f,
                    "Missing bound: {}:{} - trait {} needs {}",
                    file.display(),
                    line,
                    trait_name,
                    missing_bound
                )
            }
            Self::MissingAsyncTrait {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing #[async_trait]: {}:{} - trait {}",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::RawResultType {
                file,
                line,
                context,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Raw Result type: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    context,
                    suggestion
                )
            }
            Self::MissingInterfaceBound {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing Interface bound: {}:{} - trait {} needs : Interface",
                    file.display(),
                    line,
                    trait_name
                )
            }
        }
    }
}

impl Violation for PatternViolation {
    fn id(&self) -> &str {
        match self {
            Self::ConcreteTypeInDi { .. } => "PAT001",
            Self::MissingSendSync { .. } => "PAT002",
            Self::MissingAsyncTrait { .. } => "PAT003",
            Self::RawResultType { .. } => "PAT004",
            Self::MissingInterfaceBound { .. } => "PAT005",
        }
    }

    fn category(&self) -> ViolationCategory {
        match self {
            Self::ConcreteTypeInDi { .. } | Self::MissingInterfaceBound { .. } => {
                ViolationCategory::DependencyInjection
            }
            Self::MissingSendSync { .. } | Self::MissingAsyncTrait { .. } => {
                ViolationCategory::Async
            }
            Self::RawResultType { .. } => ViolationCategory::Quality,
        }
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ConcreteTypeInDi { severity, .. }
            | Self::MissingSendSync { severity, .. }
            | Self::MissingAsyncTrait { severity, .. }
            | Self::RawResultType { severity, .. }
            | Self::MissingInterfaceBound { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::ConcreteTypeInDi { file, .. }
            | Self::MissingSendSync { file, .. }
            | Self::MissingAsyncTrait { file, .. }
            | Self::RawResultType { file, .. }
            | Self::MissingInterfaceBound { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ConcreteTypeInDi { line, .. }
            | Self::MissingSendSync { line, .. }
            | Self::MissingAsyncTrait { line, .. }
            | Self::RawResultType { line, .. }
            | Self::MissingInterfaceBound { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::ConcreteTypeInDi { suggestion, .. } | Self::RawResultType { suggestion, .. } => {
                Some(format!("Use {suggestion}"))
            }
            Self::MissingSendSync { missing_bound, .. } => {
                Some(format!("Add {missing_bound} bounds to trait"))
            }
            Self::MissingAsyncTrait { .. } => Some("Add #[async_trait] attribute".to_string()),
            Self::MissingInterfaceBound { .. } => {
                Some("Add : Interface bound for dill DI".to_string())
            }
        }
    }
}

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
    ///
    /// # Code Smells
    /// (Complexity addressed via helper extraction)
    pub fn validate_trait_based_di(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        // Pattern to find Arc<SomeConcreteType> where SomeConcreteType doesn't start with "dyn"
        // Pattern to find Arc<SomeConcreteType> where SomeConcreteType doesn't start with "dyn"
        let arc_pattern = Regex::new(&self.rules.arc_pattern)
            .or_else(|_| Regex::new(r"Arc<([A-Z][a-zA-Z0-9_]*)>"))
            .map_err(crate::ValidationError::InvalidRegex)?;

        // Known concrete types that are OK to use directly
        let allowed_concrete = &self.rules.allowed_concrete_types;

        // Provider trait names that should use Arc<dyn ...>
        let provider_traits = &self.rules.provider_trait_suffixes;

        if allowed_concrete.is_empty() {
            // Warn or panic if config is unexpectedly empty?
            // panic!("PatternValidator: Rule CA017 missing 'allowed_concrete_types' in YAML config.");
        }

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for_each_scan_file(
                &self.config,
                Some(LanguageId::Rust),
                false,
                |entry, candidate_src_dir| {
                    let path = &entry.absolute_path;
                    if candidate_src_dir != src_dir {
                        return Ok(());
                    }

                    let content = std::fs::read_to_string(path)?;
                    let file_violations = self.check_arc_usage_in_file(
                        path,
                        &content,
                        &arc_pattern,
                        allowed_concrete,
                        provider_traits,
                    );
                    violations.extend(file_violations);
                    Ok(())
                },
            )?;
        }

        Ok(violations)
    }

    /// Helper to check for Arc usage violations in a single file
    fn check_arc_usage_in_file(
        &self,
        path: &std::path::Path,
        content: &str,
        arc_pattern: &Regex,
        allowed_concrete: &[String],
        provider_traits: &[String],
    ) -> Vec<PatternViolation> {
        let mut violations = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Check for ignore hints
            let has_ignore_hint = line.contains("mcb-validate-ignore: admin_service_concrete_type");

            for cap in arc_pattern.captures_iter(line) {
                let type_name = cap.get(1).map_or("", |m| m.as_str());

                // Skip allowed concrete types
                if allowed_concrete.iter().any(|s| s == type_name) {
                    continue;
                }

                // Skip if already using dyn (handled by different pattern)
                if line.contains(&format!("Arc<dyn {type_name}")) {
                    continue;
                }

                // Skip decorator pattern: Arc<Type<T>> (generic wrapper types)
                if line.contains(&format!("Arc<{type_name}<")) {
                    continue;
                }

                // Check if type name ends with a provider trait suffix
                let is_likely_provider = provider_traits
                    .iter()
                    .any(|suffix| type_name.ends_with(suffix));

                // Also check for common service implementation patterns
                let is_impl_suffix = type_name.ends_with("Impl")
                    || type_name.ends_with("Implementation")
                    || type_name.ends_with("Adapter");

                if is_likely_provider || is_impl_suffix {
                    // Skip if ignore hint is present
                    if has_ignore_hint {
                        continue;
                    }

                    let trait_name = if is_impl_suffix {
                        type_name
                            .trim_end_matches("Impl")
                            .trim_end_matches("Implementation")
                            .trim_end_matches("Adapter")
                    } else {
                        type_name
                    };

                    violations.push(PatternViolation::ConcreteTypeInDi {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        concrete_type: format!("Arc<{type_name}>"),
                        suggestion: format!("Arc<dyn {trait_name}>"),
                        severity: Severity::Warning,
                    });
                }
            }
        }
        violations
    }

    /// Check async traits have #[`async_trait`] and Send + Sync bounds.
    ///
    /// # Code Smells
    /// (Complexity addressed via helper extraction)
    pub fn validate_async_traits(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for_each_scan_file(
                &self.config,
                Some(LanguageId::Rust),
                false,
                |entry, candidate_src_dir| {
                    let path = &entry.absolute_path;
                    if candidate_src_dir != src_dir {
                        return Ok(());
                    }

                    let content = std::fs::read_to_string(path)?;
                    let file_violations = self.check_async_traits_in_file(path, &content);
                    violations.extend(file_violations);
                    Ok(())
                },
            )?;
        }

        Ok(violations)
    }

    /// Helper to check async trait violations in a single file
    fn check_async_traits_in_file(
        &self,
        path: &std::path::Path,
        content: &str,
    ) -> Vec<PatternViolation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Regexes are initialized in the main method or safely here if we want,
        // but since they are strictly static OnceLock, we can access them.
        // However, the main function initialized them. To be safe/clean,
        // we can re-retrieve them or assume they are init (but they might not be if new() didn't runs).
        // Actually, the main method calls get_or_init.
        // Let's just use get().expect() since validate_async_traits ensures init.
        // Or better, just re-call get_or_init.

        let trait_pattern = TRAIT_PATTERN.get_or_init(|| {
            Regex::new(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid trait pattern")
        });
        let async_fn_pattern = ASYNC_FN_PATTERN
            .get_or_init(|| Regex::new(r"async\s+fn\s+").expect("Invalid async fn pattern"));
        let send_sync_pattern = SEND_SYNC_PATTERN.get_or_init(|| {
            Regex::new(r":\s*.*Send\s*\+\s*Sync").expect("Invalid send sync pattern")
        });
        let async_trait_attr = ASYNC_TRAIT_ATTR.get_or_init(|| {
            Regex::new(r"#\[(async_trait::)?async_trait\]")
                .expect("Invalid async trait attr pattern")
        });
        let allow_async_fn_trait = ALLOW_ASYNC_FN_TRAIT.get_or_init(|| {
            Regex::new(r"#\[allow\(async_fn_in_trait\)\]")
                .expect("Invalid allow async fn trait pattern")
        });

        for (line_num, line) in lines.iter().enumerate() {
            // Find trait definitions
            if let Some(cap) = trait_pattern.captures(line) {
                let trait_name = cap.get(1).map_or("", |m| m.as_str());

                // Look ahead to see if trait has async methods
                let mut has_async_methods = false;

                if let Some((body_lines, _)) = crate::scan::extract_balanced_block(&lines, line_num)
                {
                    for subsequent_line in body_lines {
                        if async_fn_pattern.is_match(subsequent_line) {
                            has_async_methods = true;
                            break;
                        }
                    }
                }

                if has_async_methods {
                    let has_async_trait_attr = if line_num > 0 {
                        lines[..line_num].iter().rev().take(5).any(|l| {
                            async_trait_attr.is_match(l) || allow_async_fn_trait.is_match(l)
                        })
                    } else {
                        false
                    };

                    // Check if using native async trait support
                    let uses_native_async = if line_num > 0 {
                        lines[..line_num]
                            .iter()
                            .rev()
                            .take(5)
                            .any(|l| allow_async_fn_trait.is_match(l))
                    } else {
                        false
                    };

                    if !has_async_trait_attr {
                        violations.push(PatternViolation::MissingAsyncTrait {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            trait_name: trait_name.to_string(),
                            severity: Severity::Error,
                        });
                    }

                    // Check for Send + Sync bounds (skip for native async traits)
                    if !send_sync_pattern.is_match(line) && !uses_native_async {
                        violations.push(PatternViolation::MissingSendSync {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            trait_name: trait_name.to_string(),
                            missing_bound: "Send + Sync".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        violations
    }

    /// Verify consistent error type usage.
    ///
    /// # Code Smells
    /// (Complexity addressed via helper extraction)
    pub fn validate_result_types(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            // Skip manually excluded crates for result check (e.g. mcb-providers)
            if self.should_skip_result_check(&src_dir) {
                continue;
            }

            for_each_scan_file(
                &self.config,
                Some(LanguageId::Rust),
                false,
                |entry, candidate_src_dir| {
                    let path = &entry.absolute_path;
                    if candidate_src_dir != src_dir {
                        return Ok(());
                    }

                    let content = std::fs::read_to_string(path)?;
                    let file_violations = self.check_result_types_in_file(path, &content);
                    violations.extend(file_violations);
                    Ok(())
                },
            )?;
        }

        Ok(violations)
    }

    /// Helper to check result type violations in a single file
    fn check_result_types_in_file(
        &self,
        path: &std::path::Path,
        content: &str,
    ) -> Vec<PatternViolation> {
        let mut violations = Vec::new();

        // Pattern to find std::result::Result usage
        let std_result_pattern = STD_RESULT_PATTERN.get_or_init(|| {
            Regex::new(r"std::result::Result<").expect("Invalid std result pattern")
        });

        // Pattern to find Result<T, E> with explicit error type (not crate::Result)
        let explicit_result_pattern = EXPLICIT_RESULT_PATTERN.get_or_init(|| {
            Regex::new(r"Result<[^,]+,\s*([A-Za-z][A-Za-z0-9_:]+)>")
                .expect("Invalid explicit result pattern")
        });

        // Skip error-related files (they define/extend error types)
        let file_name = path.file_name().and_then(|n| n.to_str());
        let parent_name = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str());
        if file_name
            .is_some_and(|n| n == "error.rs" || n == "error_ext.rs" || n.starts_with("error"))
            || parent_name.is_some_and(|p| p == "error")
        {
            return violations;
        }

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and use statements
            if trimmed.starts_with("//") || trimmed.starts_with("use ") {
                continue;
            }

            // Check for std::result::Result
            if std_result_pattern.is_match(line) {
                violations.push(PatternViolation::RawResultType {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    context: trimmed.to_string(),
                    suggestion: "crate::Result<T>".to_string(),
                    severity: Severity::Warning,
                });
            }

            // Check for Result<T, SomeError> with explicit error type
            if let Some(cap) = explicit_result_pattern.captures(line) {
                let error_type = cap.get(1).map_or("", |m| m.as_str());

                // Allow certain standard error types
                let allowed_errors = [
                    "Error",
                    "crate::Error",
                    "crate::error::Error",
                    "ValidationError",
                    "std::io::Error",
                    "anyhow::Error",
                ];

                if !allowed_errors.contains(&error_type)
                    && !error_type.starts_with("crate::")
                    && !error_type.starts_with("self::")
                {
                    // This is informational - sometimes explicit error types are needed
                    // We won't flag this as a violation for now
                }
            }
        }
        violations
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
}

crate::impl_validator!(
    PatternValidator,
    "patterns",
    "Validates code patterns (DI, async traits, error handling)"
);
