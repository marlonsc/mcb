use std::path::{Path, PathBuf};

use derive_more::Display;

use crate::config::FileConfig;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::rules::yaml_loader::ValidatedRule;
use mcb_domain::ports::validation::{Severity, Violation, ViolationCategory};

pub(crate) fn build_substitution_variables(workspace_root: &Path) -> serde_yaml::Value {
    let file_config = FileConfig::load(workspace_root);
    let variables_val = serde_yaml::to_value(&file_config.rules.naming)
        .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    // INTENTIONAL: YAML mapping clone; empty mapping is valid default
    let mut variables = variables_val.as_mapping().cloned().unwrap_or_default();

    let ca_val = serde_yaml::to_value(&file_config.rules.clean_architecture)
        .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    if let Some(ca_map) = ca_val.as_mapping() {
        for (k, v) in ca_map {
            variables.insert(k.clone(), v.clone());
        }
    }

    let crates = [
        "domain",
        "application",
        "providers",
        "infrastructure",
        "server",
        "validate",
    ];
    for name in crates {
        let key = format!("{name}_crate");
        if let Some(val) = variables.get(serde_yaml::Value::String(key.clone()))
            && let Some(s) = val.as_str()
        {
            variables.insert(
                serde_yaml::Value::String(format!("{name}_module")),
                serde_yaml::Value::String(s.replace('-', "_")),
            );
        }
    }

    if let Some(domain_val) = variables.get(serde_yaml::Value::String("domain_crate".into()))
        && let Some(domain_str) = domain_val.as_str()
    {
        let prefix = if let Some(idx) = domain_str.find('-') {
            domain_str[0..idx].to_string()
        } else {
            domain_str.to_owned()
        };
        variables.insert(
            serde_yaml::Value::String("project_prefix".into()),
            serde_yaml::Value::String(prefix),
        );
    }

    serde_yaml::Value::Mapping(variables)
}

#[derive(Debug, Display)]
#[display("[{rule_id}] {message}")]
pub(crate) struct PatternMatchViolation {
    pub(crate) rule_id: String,
    pub(crate) file_path: PathBuf,
    pub(crate) line: usize,
    pub(crate) message: String,
    pub(crate) severity: Severity,
    pub(crate) category: ViolationCategory,
}

impl Violation for PatternMatchViolation {
    fn id(&self) -> &str {
        &self.rule_id
    }

    fn category(&self) -> ViolationCategory {
        self.category
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&PathBuf> {
        Some(&self.file_path)
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

pub(crate) fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

pub(crate) fn parse_category(s: &str) -> ViolationCategory {
    match s.to_lowercase().as_str() {
        "architecture" | "clean-architecture" => ViolationCategory::Architecture,
        "performance" => ViolationCategory::Performance,
        "testing" => ViolationCategory::Testing,
        "documentation" => ViolationCategory::Documentation,
        "naming" => ViolationCategory::Naming,
        "organization" => ViolationCategory::Organization,
        "solid" => ViolationCategory::Solid,
        "implementation" => ViolationCategory::Implementation,
        "refactoring" => ViolationCategory::Refactoring,
        _ => ViolationCategory::Quality,
    }
}

pub(crate) fn validate_path_rules(
    workspace_root: &Path,
    rules: &[ValidatedRule],
    files: &[PathBuf],
) -> Vec<Box<dyn Violation>> {
    let path_rules: Vec<&ValidatedRule> = rules
        .iter()
        .filter(|r| r.enabled && r.engine == "path")
        .collect();

    if path_rules.is_empty() {
        return Vec::new();
    }

    let filter_executor = RuleFilterExecutor::new(workspace_root.to_path_buf());
    let workspace_deps = match filter_executor.parse_workspace_dependencies() {
        Ok(deps) => deps,
        Err(e) => {
            mcb_domain::warn!(
                "validate",
                "Failed to parse workspace dependencies for path rules",
                &e.to_string()
            );
            return Vec::new();
        }
    };

    let mut violations: Vec<Box<dyn Violation>> = Vec::new();

    for rule in &path_rules {
        for file in files {
            let Some(filters) = &rule.filters else {
                continue;
            };

            let should_exec = filter_executor
                .should_execute_rule(filters, file, None, &workspace_deps)
                .unwrap_or(false);

            if should_exec {
                violations.push(Box::new(PatternMatchViolation {
                    rule_id: rule.id.clone(),
                    file_path: file.clone(),
                    line: 0,
                    message: rule.message.clone().unwrap_or_else(|| {
                        format!(
                            "[{}] File placement violation: {}",
                            rule.id, rule.description
                        )
                    }),
                    severity: parse_severity(&rule.severity),
                    category: parse_category(&rule.category),
                }));
            }
        }
    }

    violations
}
