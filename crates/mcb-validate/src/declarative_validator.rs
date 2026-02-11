//! Declarative rule validator that executes embedded YAML rules.

use std::path::PathBuf;

use anyhow::Result;

use crate::ValidationConfig;
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_rs_path;
use crate::validator_trait::Validator;
use crate::violation_trait::Violation;

/// Executes embedded YAML declarative rules against the workspace.
///
/// Currently supports rules with `metrics` configuration sections.
/// Lint-select, regex, and AST query execution are added in later phases.
pub struct DeclarativeValidator {
    /// Root directory of the workspace being validated (used by lint/regex execution slices).
    workspace_root: PathBuf,
}

impl DeclarativeValidator {
    /// Create a new declarative validator rooted at `workspace_root`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    fn load_embedded_rules(&self) -> Result<Vec<ValidatedRule>> {
        let variables = Self::build_substitution_variables(&self.workspace_root);
        let embedded = EmbeddedRules::all_yaml();
        let mut loader = YamlRuleLoader::from_embedded_with_variables(&embedded, Some(variables))?;
        let rules = loader.load_embedded_rules()?;
        Ok(rules)
    }

    fn build_substitution_variables(workspace_root: &PathBuf) -> serde_yaml::Value {
        let file_config = FileConfig::load(workspace_root);
        let variables_val = serde_yaml::to_value(&file_config.rules.naming)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        let mut variables = variables_val.as_mapping().cloned().unwrap_or_default();

        let crates = [
            "domain",
            "application",
            "providers",
            "infrastructure",
            "server",
            "validate",
            "language_support",
            "ast_utils",
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
                domain_str.to_string()
            };
            variables.insert(
                serde_yaml::Value::String("project_prefix".into()),
                serde_yaml::Value::String(prefix),
            );
        }

        serde_yaml::Value::Mapping(variables)
    }

    fn collect_rs_files(&self, config: &ValidationConfig) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let _ = for_each_scan_rs_path(config, true, |path, _src_dir| {
            files.push(path.to_path_buf());
            Ok(())
        });
        files
    }

    fn validate_metrics_rules(
        &self,
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        let metrics_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && r.metrics.is_some())
            .collect();

        if metrics_rules.is_empty() {
            return violations;
        }

        for rule in &metrics_rules {
            let Some(metrics_config) = &rule.metrics else {
                continue;
            };

            let thresholds = MetricThresholds::from_metrics_config(metrics_config);
            let analyzer = RcaAnalyzer::with_thresholds(thresholds);

            for file in files {
                if let Ok(file_violations) = analyzer.find_violations(file) {
                    let typed: Vec<MetricViolation> = file_violations;
                    violations.extend(typed.into_iter().map(|v| Box::new(v) as Box<dyn Violation>));
                }
            }
        }

        violations
    }
}

impl Validator for DeclarativeValidator {
    fn name(&self) -> &'static str {
        "declarative_rules"
    }

    fn description(&self) -> &'static str {
        "Executes embedded YAML declarative rules"
    }

    fn enabled_by_default(&self) -> bool {
        false
    }

    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let rules = self.load_embedded_rules()?;
        let files = self.collect_rs_files(config);
        let violations = self.validate_metrics_rules(&rules, &files);
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_validator_loads_rules() {
        let validator = DeclarativeValidator::new("/nonexistent");
        let rules = validator.load_embedded_rules().unwrap();
        assert!(
            !rules.is_empty(),
            "Embedded rules should load without error"
        );
    }

    #[test]
    fn test_declarative_validator_empty_workspace() {
        let dir = std::env::temp_dir().join("mcb_decl_test_empty");
        let _ = std::fs::create_dir_all(&dir);
        let validator = DeclarativeValidator::new(&dir);
        let config = ValidationConfig::new(&dir);
        let result = validator.validate(&config).unwrap();
        assert!(
            result.is_empty(),
            "Empty workspace should produce no violations"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
