//! Module Visibility Validation
//!
//! Validates proper use of pub(crate), pub, and private visibility.

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::VisibilityRulesConfig;
use crate::define_violations;
use crate::scan::for_each_file_under_root;
use crate::traits::violation::ViolationCategory;
use crate::{Result, ValidationConfig};

define_violations! {
    ViolationCategory::Organization,
    pub enum VisibilityViolation {

        /// Internal helper is public but should be restricted.
        #[violation(
            id = "VIS001",
            severity = Info,
            message = "Internal helper {item_name} is pub at {file}:{line} - consider pub(crate)",
            suggestion = "Use pub(crate) for internal helpers"
        )]
        InternalHelperTooPublic {
            item_name: String,
            file: PathBuf,
            line: usize,
        },
        /// Domain type visibility is too restricted (should be public).
        #[violation(
            id = "VIS002",
            severity = Warning,
            message = "Domain type {type_name} is pub(crate) at {file}:{line} - should be pub",
            suggestion = "Domain types should use pub for external use"
        )]
        DomainTypeTooRestricted {
            type_name: String,
            file: PathBuf,
            line: usize,
        },
        /// Utility module exposes too many public items.
        #[violation(
            id = "VIS003",
            severity = Info,
            message = "Utility module {module_name} has pub items at {file}:{line}",
            suggestion = "Consider pub(crate) for utility modules"
        )]
        UtilityModuleTooPublic {
            module_name: String,
            file: PathBuf,
            line: usize,
        },
    }
}

/// Visibility Validator
pub struct VisibilityValidator {
    internal_dirs: Vec<String>,
    exempted_items: HashSet<String>,
    utility_module_patterns: Vec<String>,
    pub_count_threshold: usize,
    scan_crates: Vec<String>,
    enabled: bool,
}

struct InternalHelperScanInput<'a> {
    content: &'a str,
    path: &'a std::path::Path,
    pub_crate_re: &'a regex::Regex,
    pub_item_re_internal: &'a regex::Regex,
    exempted_items: &'a HashSet<String>,
}

crate::impl_config_only_validator_new!(VisibilityValidator, visibility);

impl VisibilityValidator {
    /// Creates a new visibility validator with current configuration.
    #[must_use]
    pub fn with_config(config: &VisibilityRulesConfig) -> Self {
        let internal_dirs = config.internal_dirs.clone();
        let exempted_items: HashSet<String> = config.exempted_items.iter().cloned().collect();
        let utility_module_patterns = config.utility_module_patterns.clone();
        let pub_count_threshold = config.pub_count_threshold;
        let scan_crates = config.scan_crates.clone();
        let enabled = config.enabled;

        Self {
            internal_dirs,
            exempted_items,
            utility_module_patterns,
            pub_count_threshold,
            scan_crates,
            enabled,
        }
    }

    /// Validates visibility rules for the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<VisibilityViolation>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();
        violations.extend(self.check_internal_helpers(config)?);
        violations.extend(self.check_utility_modules(config)?);
        Ok(violations)
    }

    fn check_internal_helpers(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();
        let pub_crate_re = compile_regex(r"^pub\(crate\)")?;
        let pub_item_re_internal =
            compile_regex(r"^pub\s+(fn|struct|enum|type|const|static)\s+(\w+)")?;

        for dir_path in &self.internal_dirs {
            let full_path = config.workspace_root.join(dir_path);
            if !full_path.exists() {
                continue;
            }

            for_each_file_under_root(config, &full_path, Some(LanguageId::Rust), |entry| {
                let path = &entry.absolute_path;
                let content = std::fs::read_to_string(path)?;
                Self::collect_internal_helper_violations(
                    &InternalHelperScanInput {
                        content: &content,
                        path,
                        pub_crate_re: &pub_crate_re,
                        pub_item_re_internal: &pub_item_re_internal,
                        exempted_items: &self.exempted_items,
                    },
                    &mut violations,
                );

                Ok(())
            })?;
        }
        Ok(violations)
    }

    fn check_utility_modules(&self, config: &ValidationConfig) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();
        let pub_crate_re = compile_regex(r"^pub\(crate\)")?;
        let pub_item_re_utility = compile_regex(r"^pub\s+(fn|struct|enum|type)\s+(\w+)")?;

        for crate_name in &self.scan_crates {
            let crate_src = config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("src");
            if !crate_src.exists() {
                continue;
            }

            for_each_file_under_root(config, &crate_src, Some(LanguageId::Rust), |entry| {
                let path = &entry.absolute_path;
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !self.utility_module_patterns.iter().any(|p| p == file_name) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                let pub_count =
                    Self::count_utility_pub_items(&content, &pub_crate_re, &pub_item_re_utility);

                if pub_count > self.pub_count_threshold {
                    violations.push(VisibilityViolation::UtilityModuleTooPublic {
                        module_name: file_name.trim_end_matches(".rs").to_owned(),
                        file: path.clone(),
                        line: 1,
                    });
                }

                Ok(())
            })?;
        }
        Ok(violations)
    }

    fn collect_internal_helper_violations(
        input: &InternalHelperScanInput<'_>,
        violations: &mut Vec<VisibilityViolation>,
    ) {
        for (line_num, line) in input.content.lines().enumerate() {
            let trimmed = line.trim();
            if input.pub_crate_re.is_match(trimmed) {
                continue;
            }

            if let Some(captures) = input.pub_item_re_internal.captures(trimmed) {
                let item_name = captures.get(2).map_or("unknown", |m| m.as_str());
                if input.exempted_items.contains(item_name) {
                    continue;
                }

                violations.push(VisibilityViolation::InternalHelperTooPublic {
                    item_name: item_name.to_owned(),
                    file: input.path.to_path_buf(),
                    line: line_num + 1,
                });
            }
        }
    }

    fn count_utility_pub_items(
        content: &str,
        pub_crate_re: &regex::Regex,
        pub_item_re_utility: &regex::Regex,
    ) -> usize {
        content
            .lines()
            .map(str::trim)
            .filter(|trimmed| {
                !pub_crate_re.is_match(trimmed) && pub_item_re_utility.is_match(trimmed)
            })
            .count()
    }
}

/// Validator trait implementation for visibility validation.
impl crate::traits::validator::Validator for VisibilityValidator {
    fn name(&self) -> &'static str {
        "visibility"
    }

    fn description(&self) -> &'static str {
        "Validates visibility modifiers for proper encapsulation"
    }

    fn validate(
        &self,
        config: &crate::ValidationConfig,
    ) -> crate::Result<Vec<Box<dyn crate::traits::violation::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
            .collect())
    }
}
