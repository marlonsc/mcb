//! Module Visibility Validation
//!
//! Validates proper use of pub(crate), pub, and private visibility.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

use crate::config::VisibilityRulesConfig;
use crate::scan::for_each_rs_under_root;
use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationConfig};

/// Visibility Violations
#[derive(Debug, Clone, Serialize)]
pub enum VisibilityViolation {
    /// Internal helper is public but should be restricted.
    InternalHelperTooPublic {
        /// Name of the helper item.
        item_name: String,
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
    },
    /// Domain type visibility is too restricted (should be public).
    DomainTypeTooRestricted {
        /// Name of the domain type.
        type_name: String,
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
    },
    /// Utility module exposes too many public items.
    UtilityModuleTooPublic {
        /// Name of the utility module.
        module_name: String,
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
    },
}

/// Display implementation for visibility violations.
impl std::fmt::Display for VisibilityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalHelperTooPublic {
                item_name,
                file,
                line,
            } => write!(
                f,
                "Internal helper {} is pub at {}:{} - consider pub(crate)",
                item_name,
                file.display(),
                line
            ),
            Self::DomainTypeTooRestricted {
                type_name,
                file,
                line,
            } => write!(
                f,
                "Domain type {} is pub(crate) at {}:{} - should be pub",
                type_name,
                file.display(),
                line
            ),
            Self::UtilityModuleTooPublic {
                module_name,
                file,
                line,
            } => write!(
                f,
                "Utility module {} has pub items at {}:{}",
                module_name,
                file.display(),
                line
            ),
        }
    }
}

/// Violation trait implementation for visibility violations.
impl Violation for VisibilityViolation {
    fn id(&self) -> &str {
        match self {
            Self::InternalHelperTooPublic { .. } => "VIS001",
            Self::DomainTypeTooRestricted { .. } => "VIS002",
            Self::UtilityModuleTooPublic { .. } => "VIS003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Organization
    }

    fn severity(&self) -> Severity {
        match self {
            Self::InternalHelperTooPublic { .. } | Self::UtilityModuleTooPublic { .. } => {
                Severity::Info
            }
            Self::DomainTypeTooRestricted { .. } => Severity::Warning,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::InternalHelperTooPublic { file, .. }
            | Self::DomainTypeTooRestricted { file, .. }
            | Self::UtilityModuleTooPublic { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::InternalHelperTooPublic { line, .. }
            | Self::DomainTypeTooRestricted { line, .. }
            | Self::UtilityModuleTooPublic { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::InternalHelperTooPublic { .. } => {
                Some("Use pub(crate) for internal helpers".to_string())
            }
            Self::DomainTypeTooRestricted { .. } => {
                Some("Domain types should use pub for external use".to_string())
            }
            Self::UtilityModuleTooPublic { .. } => {
                Some("Consider pub(crate) for utility modules".to_string())
            }
        }
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

fn pub_item_re_internal() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^pub\s+(fn|struct|enum|type|const|static)\s+(\w+)")
            .expect("Invalid internal visibility regex")
    })
}

fn pub_item_re_utility() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^pub\s+(fn|struct|enum|type)\s+(\w+)")
            .expect("Invalid utility visibility regex")
    })
}

fn pub_crate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^pub\(crate\)").expect("Invalid pub(crate) regex"))
}

impl VisibilityValidator {
    /// Creates a new visibility validator, loading configuration from files.
    pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        let file_config = crate::config::FileConfig::load(workspace_root);
        Self::with_config(&file_config.rules.visibility)
    }

    /// Creates a new visibility validator with current configuration.
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

        for dir_path in &self.internal_dirs {
            let full_path = config.workspace_root.join(dir_path);
            if !full_path.exists() {
                continue;
            }

            for_each_rs_under_root(config, &full_path, |path| {
                let content = std::fs::read_to_string(path)?;
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if pub_crate_re().is_match(trimmed) {
                        continue;
                    }

                    if let Some(captures) = pub_item_re_internal().captures(trimmed) {
                        let item_name = captures.get(2).map_or("unknown", |m| m.as_str());
                        // Skip exempted items:
                        if self.exempted_items.contains(item_name) {
                            continue;
                        }

                        violations.push(VisibilityViolation::InternalHelperTooPublic {
                            item_name: item_name.to_string(),
                            file: path.to_path_buf(),
                            line: line_num + 1,
                        });
                    }
                }

                Ok(())
            })?;
        }
        Ok(violations)
    }

    fn check_utility_modules(&self, config: &ValidationConfig) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();

        for crate_name in &self.scan_crates {
            let crate_src = config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("src");
            if !crate_src.exists() {
                continue;
            }

            for_each_rs_under_root(config, &crate_src, |path| {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !self.utility_module_patterns.iter().any(|p| p == file_name) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                let mut pub_count = 0;

                for line in content.lines() {
                    let trimmed = line.trim();
                    if pub_crate_re().is_match(trimmed) {
                        continue;
                    }
                    if pub_item_re_utility().is_match(trimmed) {
                        pub_count += 1;
                    }
                }

                if pub_count > self.pub_count_threshold {
                    violations.push(VisibilityViolation::UtilityModuleTooPublic {
                        module_name: file_name.trim_end_matches(".rs").to_string(),
                        file: path.to_path_buf(),
                        line: 1,
                    });
                }

                Ok(())
            })?;
        }
        Ok(violations)
    }
}

/// Validator trait implementation for visibility validation.
impl crate::validator_trait::Validator for VisibilityValidator {
    fn name(&self) -> &'static str {
        "visibility"
    }

    fn description(&self) -> &'static str {
        "Validates visibility modifiers for proper encapsulation"
    }

    fn validate(
        &self,
        config: &crate::ValidationConfig,
    ) -> anyhow::Result<Vec<Box<dyn crate::violation_trait::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::violation_trait::Violation>)
            .collect())
    }
}
