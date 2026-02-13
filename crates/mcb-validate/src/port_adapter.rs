//! Port/Adapter Compliance Validation
//!
//! Validates Clean Architecture port/adapter patterns.

use std::path::PathBuf;

use regex::Regex;
use serde::Serialize;

use crate::config::PortAdapterRulesConfig;
use crate::scan::for_each_rs_under_root;
use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationConfig};

/// Port/Adapter Violations
#[derive(Debug, Clone, Serialize)]
pub enum PortAdapterViolation {
    /// Adapter lacks a corresponding port implementation
    AdapterMissingPortImpl {
        /// Name of the adapter that is missing an implementation of its corresponding port.
        adapter_name: String,
        /// File where the adapter is defined.
        file: PathBuf,
        /// Line number where the adapter definition begins.
        line: usize,
    },
    /// Adapter depends on another concrete adapter instead of a port
    AdapterUsesAdapter {
        /// Name of the adapter that contains the violation.
        adapter_name: String,
        /// Name of the concrete adapter being incorrectly referenced.
        other_adapter: String,
        /// File where the incorrect usage occurred.
        file: PathBuf,
        /// Line number where the incorrect usage occurred.
        line: usize,
    },
    /// Port has too many methods (violates ISP)
    PortTooLarge {
        /// Name of the port (trait) that has too many methods.
        trait_name: String,
        /// The total number of methods found in the port.
        method_count: usize,
        /// File where the port is defined.
        file: PathBuf,
        /// Line number where the port definition begins.
        line: usize,
    },
    /// Port has too few methods (may indicate over-fragmentation)
    PortTooSmall {
        /// Name of the port (trait) that has too few methods.
        trait_name: String,
        /// The total number of methods found in the port.
        method_count: usize,
        /// File where the port is defined.
        file: PathBuf,
        /// Line number where the port definition begins.
        line: usize,
    },
}

impl std::fmt::Display for PortAdapterViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdapterMissingPortImpl {
                adapter_name,
                file,
                line,
            } => write!(
                f,
                "Adapter {} missing port impl at {}:{}",
                adapter_name,
                file.display(),
                line
            ),
            Self::AdapterUsesAdapter {
                adapter_name,
                other_adapter,
                file,
                line,
            } => write!(
                f,
                "Adapter {} uses {} directly at {}:{}",
                adapter_name,
                other_adapter,
                file.display(),
                line
            ),
            Self::PortTooLarge {
                trait_name,
                method_count,
                file,
                line,
            } => write!(
                f,
                "Port {} has {} methods (>10) at {}:{}",
                trait_name,
                method_count,
                file.display(),
                line
            ),
            Self::PortTooSmall {
                trait_name,
                method_count,
                file,
                line,
            } => write!(
                f,
                "Port {} has {} method(s) at {}:{}",
                trait_name,
                method_count,
                file.display(),
                line
            ),
        }
    }
}

impl Violation for PortAdapterViolation {
    fn id(&self) -> &str {
        match self {
            Self::AdapterMissingPortImpl { .. } => "PORT001",
            Self::AdapterUsesAdapter { .. } => "PORT002",
            Self::PortTooLarge { .. } => "PORT003",
            Self::PortTooSmall { .. } => "PORT004",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::AdapterMissingPortImpl { .. } | Self::AdapterUsesAdapter { .. } => {
                Severity::Warning
            }
            Self::PortTooLarge { .. } | Self::PortTooSmall { .. } => Severity::Info,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::AdapterMissingPortImpl { file, .. }
            | Self::AdapterUsesAdapter { file, .. }
            | Self::PortTooLarge { file, .. }
            | Self::PortTooSmall { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::AdapterMissingPortImpl { line, .. }
            | Self::AdapterUsesAdapter { line, .. }
            | Self::PortTooLarge { line, .. }
            | Self::PortTooSmall { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::AdapterMissingPortImpl { .. } => {
                Some("Implement a port trait from mcb-application/ports/".to_string())
            }
            Self::AdapterUsesAdapter { .. } => {
                Some("Depend on port traits, not concrete adapters".to_string())
            }
            Self::PortTooLarge { .. } => {
                Some("Consider splitting into smaller interfaces (ISP)".to_string())
            }
            Self::PortTooSmall { .. } => Some("May indicate over-fragmentation".to_string()),
        }
    }
}

/// Validates Clean Architecture port/adapter patterns and interface segregation.
pub struct PortAdapterValidator {
    max_port_methods: usize,
    adapter_suffixes: Vec<String>,
    ports_dir: String,
    providers_dir: String,
}

impl PortAdapterValidator {
    /// Creates a new port/adapter validator, loading configuration from files.
    pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        let file_config = crate::config::FileConfig::load(workspace_root);
        Self::with_config(&file_config.rules.port_adapter)
    }

    /// Creates a new port/adapter validator with current configuration.
    pub fn with_config(config: &PortAdapterRulesConfig) -> Self {
        Self {
            max_port_methods: config.max_port_methods,
            adapter_suffixes: config.adapter_suffixes.clone(),
            ports_dir: config.ports_dir.clone(),
            providers_dir: config.providers_dir.clone(),
        }
    }

    /// Validates port/adapter compliance for the given configuration.
    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.check_port_trait_sizes(config)?);
        violations.extend(self.check_adapter_direct_usage(config)?);
        Ok(violations)
    }

    fn check_port_trait_sizes(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        let ports_dir = config.workspace_root.join(&self.ports_dir);
        if !ports_dir.exists() {
            return Ok(violations);
        }

        let trait_start_re = Regex::new(r"pub\s+trait\s+(\w+)").unwrap();
        let fn_re = Regex::new(r"^\s*(?:async\s+)?fn\s+\w+").unwrap();

        for_each_rs_under_root(config, &ports_dir, |path| {
            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut current_trait: Option<(String, usize, usize)> = None;
            let mut brace_depth = 0;
            let mut in_trait = false;

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(captures) = trait_start_re.captures(line) {
                    let trait_name = captures
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .expect("Trait regex should capture group 1");
                    current_trait = Some((trait_name, line_num + 1, 0));
                    in_trait = true;
                }

                if in_trait {
                    brace_depth += line.matches('{').count();
                    brace_depth -= line.matches('}').count();

                    if fn_re.is_match(line)
                        && let Some((_, _, ref mut count)) = current_trait
                    {
                        *count += 1;
                    }

                    if brace_depth == 0 && current_trait.is_some() {
                        let (trait_name, start_line, method_count) = current_trait
                            .take()
                            .expect("current_trait should exist when brace_depth == 0");
                        in_trait = false;

                        // Flag ports with too many methods (violates ISP)
                        // Single-method interfaces are valid per ISP - don't flag them
                        if method_count > self.max_port_methods {
                            violations.push(PortAdapterViolation::PortTooLarge {
                                trait_name,
                                method_count,
                                file: path.to_path_buf(),
                                line: start_line,
                            });
                        }
                    }
                }
            }

            Ok(())
        })?;
        Ok(violations)
    }

    fn check_adapter_direct_usage(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        let providers_dir = config.workspace_root.join(&self.providers_dir);
        if !providers_dir.exists() {
            return Ok(violations);
        }

        let adapter_import_re = Regex::new(
            r"use\s+(?:crate|super)::(?:\w+::)*(\w+(?:Provider|Repository|Adapter|Client))",
        )
        .unwrap();

        for_each_rs_under_root(config, &providers_dir, |path| {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "mod.rs" || file_name == "lib.rs" {
                return Ok(());
            }
            // Skip test files: adapter tests legitimately use the concrete type
            if file_name == "tests.rs"
                || path
                    .parent()
                    .and_then(|p| p.file_name())
                    .is_some_and(|n| n == "tests")
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let current_adapter = file_name.trim_end_matches(".rs");

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                if let Some(captures) = adapter_import_re.captures(line) {
                    let imported = captures.get(1).map_or("", |m| m.as_str());
                    if imported.to_lowercase().contains(current_adapter) {
                        continue;
                    }

                    for suffix in &self.adapter_suffixes {
                        if imported.ends_with(suffix) && !imported.starts_with("dyn") {
                            violations.push(PortAdapterViolation::AdapterUsesAdapter {
                                adapter_name: current_adapter.to_string(),
                                other_adapter: imported.to_string(),
                                file: path.to_path_buf(),
                                line: line_num + 1,
                            });
                            break;
                        }
                    }
                }
            }

            Ok(())
        })?;
        Ok(violations)
    }
}

impl crate::validator_trait::Validator for PortAdapterValidator {
    fn name(&self) -> &'static str {
        "port_adapter"
    }

    fn description(&self) -> &'static str {
        "Validates port/adapter patterns for Clean Architecture compliance"
    }

    fn validate(
        &self,
        config: &ValidationConfig,
    ) -> anyhow::Result<Vec<Box<dyn crate::violation_trait::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::violation_trait::Violation>)
            .collect())
    }
}
