//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! YAML Rule Loader
//!
//! Automatically loads and validates YAML-based rules with template support.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde_yaml;

use super::templates::TemplateEngine;
use super::yaml_validator::YamlRuleValidator;
use crate::Result;

pub use super::rule_types::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule,
};
use crate::utils::fs::collect_yaml_files;
use mcb_utils::constants::validate::{
    DEFAULT_RULE_CATEGORY, DEFAULT_RULE_DESCRIPTION, DEFAULT_RULE_NAME, DEFAULT_RULE_RATIONALE,
    RUSTY_RULES, SEVERITY_WARNING, YAML_FIELD_AST_QUERY, YAML_FIELD_BASE, YAML_FIELD_CATEGORY,
    YAML_FIELD_CONFIG, YAML_FIELD_DESCRIPTION, YAML_FIELD_ENABLED, YAML_FIELD_ENGINE,
    YAML_FIELD_EXTENDS, YAML_FIELD_FILTERS, YAML_FIELD_FIX_TYPE, YAML_FIELD_FIXES, YAML_FIELD_ID,
    YAML_FIELD_LANGUAGE, YAML_FIELD_LINT_SELECT, YAML_FIELD_MESSAGE, YAML_FIELD_METRICS,
    YAML_FIELD_NAME, YAML_FIELD_NODE_TYPE, YAML_FIELD_PATTERN, YAML_FIELD_RATIONALE,
    YAML_FIELD_RULE, YAML_FIELD_SELECTORS, YAML_FIELD_SEVERITY, YAML_FIELD_TEMPLATE,
};

/// YAML rule loader with automatic discovery
pub struct YamlRuleLoader {
    /// Validator for checking YAML syntax against schema
    validator: YamlRuleValidator,
    /// Engine for processing rule templates and inheritance
    template_engine: TemplateEngine,
    /// Directory containing the rule definitions
    rules_dir: PathBuf,
    /// Variables for template substitution (e.g. from config)
    variables: Option<serde_yaml::Value>,
    embedded_rules: Option<Vec<(String, String)>>,
}

impl YamlRuleLoader {
    fn is_template_path(path: &str) -> bool {
        path.contains("/templates/")
            || path.starts_with("templates/")
            || path.contains("\\templates\\")
    }

    /// Create a new YAML rule loader
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML schema validator cannot be initialized.
    pub fn new(rules_dir: PathBuf) -> Result<Self> {
        Self::with_variables(rules_dir, None)
    }

    /// Create a new YAML rule loader with variables
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML schema validator cannot be initialized.
    pub fn with_variables(
        rules_dir: PathBuf,
        variables: Option<serde_yaml::Value>,
    ) -> Result<Self> {
        Ok(Self {
            validator: YamlRuleValidator::new()?,
            template_engine: TemplateEngine::new(),
            rules_dir,
            variables,
            embedded_rules: None,
        })
    }

    /// Create a YAML loader backed by embedded `(path, content)` entries.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML schema validator cannot be initialized.
    pub fn from_embedded(rules: &[(&str, &str)]) -> Result<Self> {
        Self::from_embedded_with_variables(rules, None)
    }

    /// Create a YAML loader backed by embedded entries with substitution variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML schema validator cannot be initialized.
    pub fn from_embedded_with_variables(
        rules: &[(&str, &str)],
        variables: Option<serde_yaml::Value>,
    ) -> Result<Self> {
        Ok(Self {
            validator: YamlRuleValidator::new()?,
            template_engine: TemplateEngine::new(),
            rules_dir: PathBuf::new(),
            variables,
            embedded_rules: Some(
                rules
                    .iter()
                    .map(|(path, content)| ((*path).to_owned(), (*content).to_owned()))
                    .collect(),
            ),
        })
    }

    /// Set embedded rules for the loader.
    pub fn set_embedded_rules(&mut self, rules: Vec<(&str, &str)>) {
        self.embedded_rules = Some(
            rules
                .into_iter()
                .map(|(path, content)| (path.to_owned(), content.to_owned()))
                .collect(),
        );
    }

    /// Load all rules from embedded entries without filesystem access.
    ///
    /// # Errors
    ///
    /// Returns an error if template loading or rule parsing fails.
    pub fn load_embedded_rules(&mut self) -> Result<Vec<ValidatedRule>> {
        let mut rules = Vec::new();

        if let Some(embedded_rules) = &self.embedded_rules {
            self.template_engine
                .load_templates_from_embedded(embedded_rules)?;

            for (path, content) in embedded_rules {
                if path.ends_with(".yml") && !Self::is_template_path(path) {
                    let loaded_rules = self.load_rule_from_str(Path::new(path), content)?;
                    rules.extend(loaded_rules);
                }
            }
        }

        Ok(rules)
    }

    /// Synchronous variant of [`Self::load_all_rules`].
    ///
    /// # Errors
    ///
    /// Returns an error if template loading, file reading, or rule parsing fails.
    pub fn load_all_rules_sync(&mut self) -> Result<Vec<ValidatedRule>> {
        let mut rules = Vec::new();

        if self.embedded_rules.is_some() {
            rules.extend(self.load_embedded_rules()?);
        }

        if self.rules_dir.exists() {
            self.template_engine.load_templates_sync(&self.rules_dir)?;

            let rule_files: Vec<PathBuf> = collect_yaml_files(&self.rules_dir)?
                .into_iter()
                .filter(|path| Self::is_rule_file(path))
                .collect();

            let loaded: Result<Vec<Vec<ValidatedRule>>> = rule_files
                .par_iter()
                .map(|path| {
                    let content =
                        std::fs::read_to_string(path).map_err(crate::ValidationError::Io)?;
                    self.load_rule_from_str(path, &content)
                })
                .collect();

            for loaded_rules in loaded? {
                rules.extend(loaded_rules);
            }
        }

        Ok(rules)
    }

    /// Load all rules from the rules directory
    ///
    /// # Errors
    ///
    /// Returns an error if template loading, file reading, or rule parsing fails.
    pub async fn load_all_rules(&mut self) -> Result<Vec<ValidatedRule>> {
        if let Some(embedded_rules) = &self.embedded_rules
            && !embedded_rules.is_empty()
        {
            return self.load_embedded_rules();
        }

        let mut rules = Vec::new();

        // Load templates first
        self.template_engine.load_templates(&self.rules_dir).await?;

        // Load rule files
        for path in collect_yaml_files(&self.rules_dir)? {
            if Self::is_rule_file(&path) {
                let loaded_rules = self.load_rule_file(&path).await?;
                rules.extend(loaded_rules);
            }
        }

        Ok(rules)
    }

    /// Load rules from a specific file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or YAML parsing fails.
    pub async fn load_rule_file(&self, path: &Path) -> Result<Vec<ValidatedRule>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(crate::ValidationError::Io)?;

        self.load_rule_from_str(path, &content)
    }

    fn load_rule_from_str(&self, path: &Path, content: &str) -> Result<Vec<ValidatedRule>> {
        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("YAML parse error: {e}"),
            })?;

        // Check if this is a template
        if yaml_value
            .get(YAML_FIELD_BASE)
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false)
        {
            // This is a template, skip it
            return Ok(vec![]);
        }

        // Apply template, resolve `extends`, then substitute globals.
        let processed_yaml = self.apply_template_if_specified(yaml_value)?;
        let final_yaml = self.finalize_yaml(processed_yaml)?;

        // Convert to JSON for validating
        let json_value: serde_json::Value =
            serde_json::to_value(final_yaml).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("YAML to JSON conversion error: {e}"),
            })?;

        // Validate against schema
        self.validator.validate_rule(&json_value)?;

        // Convert to validated rule
        let validated_rule = Self::yaml_to_validated_rule(&json_value)?;

        Ok(vec![validated_rule])
    }

    /// Resolve an `extends` parent and substitute global variables into the rule YAML.
    fn finalize_yaml(&self, processed_yaml: serde_yaml::Value) -> Result<serde_yaml::Value> {
        let mut final_yaml = if let Some(extends_name) = processed_yaml
            .get(YAML_FIELD_EXTENDS)
            .and_then(|v| v.as_str())
        {
            self.template_engine
                .extend_rule(extends_name, &processed_yaml)?
        } else {
            processed_yaml
        };

        if let Some(vars) = &self.variables {
            self.template_engine
                .substitute_variables(&mut final_yaml, vars)?;
        }

        Ok(final_yaml)
    }

    /// Apply a named template to `yaml_value` when it declares one, merging globals as args.
    fn apply_template_if_specified(
        &self,
        yaml_value: serde_yaml::Value,
    ) -> Result<serde_yaml::Value> {
        let Some(template_name) = yaml_value.get(YAML_FIELD_TEMPLATE).and_then(|v| v.as_str())
        else {
            return Ok(yaml_value);
        };

        // Merge variables into rule definition so template specific logic can access them
        let template_args = if let Some(vars) = &self.variables {
            let mut args = vars.clone();
            // Simple shallow merge of rule over variables (for template arguments)
            if let serde_yaml::Value::Mapping(args_map) = &mut args
                && let serde_yaml::Value::Mapping(rule_map) = &yaml_value
            {
                for (k, v) in rule_map {
                    args_map.insert(k.clone(), v.clone());
                }
            }
            args
        } else {
            yaml_value.clone()
        };

        self.template_engine
            .apply_template(template_name, &template_args)
    }

    /// Check if a file is a rule file
    fn is_rule_file(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("yml")
            && !path.to_str().is_some_and(Self::is_template_path)
    }

    /// Convert YAML/JSON value to `ValidatedRule`
    fn yaml_to_validated_rule(value: &serde_json::Value) -> Result<ValidatedRule> {
        let obj = value
            .as_object()
            .ok_or_else(|| crate::ValidationError::Config("Rule must be an object".to_owned()))?;

        let id = obj
            .get(YAML_FIELD_ID)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config("Rule must have an 'id' field".to_owned())
            })?
            .to_owned();

        let str_field = |key: &str, default: &str| {
            obj.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or(default)
                .to_owned()
        };
        let opt_str = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(str::to_owned);
        let json_or_empty = |key: &str| {
            obj.get(key)
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        };

        Ok(ValidatedRule {
            id,
            name: str_field(YAML_FIELD_NAME, DEFAULT_RULE_NAME),
            category: str_field(YAML_FIELD_CATEGORY, DEFAULT_RULE_CATEGORY),
            severity: str_field(YAML_FIELD_SEVERITY, SEVERITY_WARNING),
            enabled: obj
                .get(YAML_FIELD_ENABLED)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            description: str_field(YAML_FIELD_DESCRIPTION, DEFAULT_RULE_DESCRIPTION),
            rationale: str_field(YAML_FIELD_RATIONALE, DEFAULT_RULE_RATIONALE),
            engine: str_field(YAML_FIELD_ENGINE, RUSTY_RULES),
            config: json_or_empty(YAML_FIELD_CONFIG),
            rule_definition: json_or_empty(YAML_FIELD_RULE),
            fixes: Self::parse_fixes(obj),
            lint_select: Self::parse_lint_select(obj),
            message: opt_str(YAML_FIELD_MESSAGE),
            selectors: Self::parse_selectors(obj),
            ast_query: opt_str(YAML_FIELD_AST_QUERY),
            metrics: Self::parse_json_field(obj, YAML_FIELD_METRICS),
            filters: Self::parse_json_field(obj, YAML_FIELD_FILTERS),
        })
    }

    /// Deserialize an optional typed sub-object field, ignoring deserialization errors.
    fn parse_json_field<T: serde::de::DeserializeOwned>(
        obj: &serde_json::Map<String, serde_json::Value>,
        key: &str,
    ) -> Option<T> {
        obj.get(key)
            .and_then(|v| serde_json::from_value::<T>(v.clone()).ok())
    }

    /// Parse the `lint_select` array of a rule object.
    fn parse_lint_select(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
        obj.get(YAML_FIELD_LINT_SELECT)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|code| code.as_str().map(str::to_owned))
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default()
    }

    /// Parse the `fixes` array of a rule object.
    fn parse_fixes(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<RuleFix> {
        obj.get(YAML_FIELD_FIXES)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|fix| {
                        let fix_obj = fix.as_object()?;
                        Some(RuleFix {
                            fix_type: fix_obj.get(YAML_FIELD_FIX_TYPE)?.as_str()?.to_owned(),
                            pattern: fix_obj
                                .get(YAML_FIELD_PATTERN)
                                .and_then(|v| v.as_str())
                                .map(str::to_owned),
                            message: fix_obj.get(YAML_FIELD_MESSAGE)?.as_str()?.to_owned(),
                        })
                    })
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default()
    }

    /// Parse the `selectors` array of a rule object.
    fn parse_selectors(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<AstSelector> {
        obj.get(YAML_FIELD_SELECTORS)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|sel| {
                        let sel_obj = sel.as_object()?;
                        Some(AstSelector {
                            language: sel_obj.get(YAML_FIELD_LANGUAGE)?.as_str()?.to_owned(),
                            node_type: sel_obj.get(YAML_FIELD_NODE_TYPE)?.as_str()?.to_owned(),
                            pattern: sel_obj
                                .get(YAML_FIELD_PATTERN)
                                .and_then(|v| v.as_str())
                                .map(str::to_owned),
                        })
                    })
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default()
    }

    /// Get rule file path for a rule ID
    #[must_use]
    pub fn get_rule_path(&self, rule_id: &str) -> Option<PathBuf> {
        // This would need a more sophisticated mapping
        // For now, just search in the rules directory
        for path in collect_yaml_files(&self.rules_dir).ok()? {
            if Self::is_rule_file(&path)
                && let Ok(content) = std::fs::read_to_string(&path)
                && content.contains(&format!("id: {rule_id}"))
            {
                return Some(path.clone());
            }
        }
        None
    }
}
