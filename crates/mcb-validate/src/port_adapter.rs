//! Port/Adapter Compliance Validation
//!
//! Validates Clean Architecture port/adapter patterns.

use std::path::PathBuf;

use regex::Regex;

use crate::config::PortAdapterRulesConfig;
use crate::scan::for_each_rs_under_root;
use crate::violation_trait::ViolationCategory;
use crate::{Result, ValidationConfig};

define_violations! {
    no_display,
    ViolationCategory::Architecture,
    pub enum PortAdapterViolation {
        /// Adapter lacks a corresponding port implementation
        #[violation(
            id = "PORT001",
            severity = Warning,
            suggestion = "Implement a port trait from mcb-application/ports/"
        )]
        AdapterMissingPortImpl {
            adapter_name: String,
            file: PathBuf,
            line: usize,
        },
        /// Adapter depends on another concrete adapter instead of a port
        #[violation(
            id = "PORT002",
            severity = Warning,
            suggestion = "Depend on port traits, not concrete adapters"
        )]
        AdapterUsesAdapter {
            adapter_name: String,
            other_adapter: String,
            file: PathBuf,
            line: usize,
        },
        /// Port has too many methods (violates ISP)
        #[violation(
            id = "PORT003",
            severity = Info,
            suggestion = "Consider splitting into smaller interfaces (ISP)"
        )]
        PortTooLarge {
            trait_name: String,
            method_count: usize,
            file: PathBuf,
            line: usize,
        },
        /// Port has too few methods (may indicate over-fragmentation)
        #[violation(
            id = "PORT004",
            severity = Info,
            suggestion = "May indicate over-fragmentation"
        )]
        PortTooSmall {
            trait_name: String,
            method_count: usize,
            file: PathBuf,
            line: usize,
        },
    }
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

        // TODO(QUAL001): unwrap() in production. Use ? or match.
        let trait_start_re = Regex::new(r"pub\s+trait\s+(\w+)").unwrap();
        // TODO(QUAL001): unwrap() in production. Use ? or match.
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
                        // TODO(QUAL002): expect() in production. Use ? or handle error.
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
                            // TODO(QUAL002): expect() in production.
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
        // TODO(QUAL001): unwrap() in production.
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
