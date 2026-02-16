//! Port/Adapter Compliance Validation
//!
//! Validates Clean Architecture port/adapter patterns.

use crate::constants::common::COMMENT_PREFIX;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use regex::Regex;
use std::path::Path;
use std::path::PathBuf;

use crate::config::PortAdapterRulesConfig;
use crate::define_violations;
use crate::scan::for_each_file_under_root;
use crate::traits::violation::ViolationCategory;
use crate::{Result, ValidationConfig};

define_violations! {
    ViolationCategory::Architecture,
    pub enum PortAdapterViolation {
        /// Adapter lacks a corresponding port implementation
        #[violation(
            id = "PORT001",
            severity = Warning,
            message = "Adapter {adapter_name} missing port impl at {file}:{line}",
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
            message = "Adapter {adapter_name} uses {other_adapter} directly at {file}:{line}",
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
            message = "Port {trait_name} has {method_count} methods (>10) at {file}:{line}",
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
            message = "Port {trait_name} has {method_count} method(s) at {file}:{line}",
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

/// Validates Clean Architecture port/adapter patterns and interface segregation.
pub struct PortAdapterValidator {
    max_port_methods: usize,
    adapter_suffixes: Vec<String>,
    ports_dir: String,
    providers_dir: String,
}

crate::impl_config_only_validator_new!(PortAdapterValidator, port_adapter);

impl PortAdapterValidator {
    /// Creates a new port/adapter validator with current configuration.
    #[must_use]
    pub fn with_config(config: &PortAdapterRulesConfig) -> Self {
        Self {
            max_port_methods: config.max_port_methods,
            adapter_suffixes: config.adapter_suffixes.clone(),
            ports_dir: config.ports_dir.clone(),
            providers_dir: config.providers_dir.clone(),
        }
    }

    /// Validates port/adapter compliance for the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
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

        let trait_start_re = compile_regex(r"pub\s+trait\s+(\w+)")?;
        let fn_re = compile_regex(r"^\s*(?:async\s+)?fn\s+\w+")?;

        for_each_file_under_root(config, &ports_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            let content = std::fs::read_to_string(path)?;
            violations.extend(collect_port_size_violations(
                path,
                &content,
                &trait_start_re,
                &fn_re,
                self.max_port_methods,
            ));

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

        let adapter_import_re = compile_regex(
            r"use\s+(?:crate|super)::(?:\w+::)*(\w+(?:Provider|Repository|Adapter|Client))",
        )?;

        for_each_file_under_root(config, &providers_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            if should_skip_provider_file(path) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            violations.extend(collect_adapter_usage_violations(
                path,
                &content,
                &adapter_import_re,
                &self.adapter_suffixes,
            ));

            Ok(())
        })?;
        Ok(violations)
    }
}

fn collect_port_size_violations(
    path: &Path,
    content: &str,
    trait_start_re: &Regex,
    fn_re: &Regex,
    max_port_methods: usize,
) -> Vec<PortAdapterViolation> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();
    let mut current_trait: Option<(String, usize, usize)> = None;
    let mut brace_depth = 0;
    let mut in_trait = false;

    for (line_num, line) in lines.iter().enumerate() {
        if let Some(trait_name) = capture_trait_name(trait_start_re, line) {
            current_trait = Some((trait_name, line_num + 1, 0));
            in_trait = true;
        }

        if !in_trait {
            continue;
        }

        brace_depth += line.matches('{').count();
        brace_depth -= line.matches('}').count();

        if fn_re.is_match(line)
            && let Some((_, _, ref mut count)) = current_trait
        {
            *count += 1;
        }

        if brace_depth != 0 {
            continue;
        }

        let Some((trait_name, start_line, method_count)) = current_trait.take() else {
            continue;
        };
        in_trait = false;

        if method_count > max_port_methods {
            violations.push(PortAdapterViolation::PortTooLarge {
                trait_name,
                method_count,
                file: path.to_path_buf(),
                line: start_line,
            });
        }
    }

    violations
}

fn capture_trait_name(trait_start_re: &Regex, line: &str) -> Option<String> {
    trait_start_re
        .captures(line)
        .and_then(|captures| captures.get(1).map(|m| m.as_str().to_owned()))
}

fn should_skip_provider_file(path: &std::path::Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "tests.rs" {
        return true;
    }

    path.parent()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n == "tests")
}

fn collect_adapter_usage_violations(
    path: &Path,
    content: &str,
    adapter_import_re: &Regex,
    adapter_suffixes: &[String],
) -> Vec<PortAdapterViolation> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let current_adapter = file_name.trim_end_matches(".rs").to_owned();
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        let Some(imported) = capture_imported_adapter(adapter_import_re, line) else {
            continue;
        };

        if imported.to_lowercase().contains(&current_adapter) {
            continue;
        }

        if !is_forbidden_adapter_import(&imported, adapter_suffixes) {
            continue;
        }

        violations.push(PortAdapterViolation::AdapterUsesAdapter {
            adapter_name: current_adapter.clone(),
            other_adapter: imported,
            file: path.to_path_buf(),
            line: line_num + 1,
        });
    }

    violations
}

fn capture_imported_adapter(adapter_import_re: &Regex, line: &str) -> Option<String> {
    adapter_import_re
        .captures(line)
        .and_then(|captures| captures.get(1).map(|m| m.as_str().to_owned()))
}

fn is_forbidden_adapter_import(imported: &str, adapter_suffixes: &[String]) -> bool {
    if imported.starts_with("dyn") {
        return false;
    }

    adapter_suffixes
        .iter()
        .any(|suffix| imported.ends_with(suffix))
}

impl crate::traits::validator::Validator for PortAdapterValidator {
    fn name(&self) -> &'static str {
        "port_adapter"
    }

    fn description(&self) -> &'static str {
        "Validates port/adapter patterns for Clean Architecture compliance"
    }

    fn validate(
        &self,
        config: &ValidationConfig,
    ) -> crate::Result<Vec<Box<dyn crate::traits::violation::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
            .collect())
    }
}
