//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! YAML Rule Loader
//!
//! Automatically loads and validates YAML-based rules with template support.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml;

use super::templates::TemplateEngine;
use super::yaml_validator::YamlRuleValidator;
use crate::Result;
use crate::constants::rules::{
    DEFAULT_RULE_CATEGORY, DEFAULT_RULE_DESCRIPTION, DEFAULT_RULE_ENGINE, DEFAULT_RULE_NAME,
    DEFAULT_RULE_RATIONALE, DEFAULT_RULE_SEVERITY, YAML_FIELD_AST_QUERY, YAML_FIELD_BASE,
    YAML_FIELD_CATEGORY, YAML_FIELD_CONFIG, YAML_FIELD_DESCRIPTION, YAML_FIELD_ENABLED,
    YAML_FIELD_ENGINE, YAML_FIELD_EXTENDS, YAML_FIELD_FILTERS, YAML_FIELD_FIX_TYPE,
    YAML_FIELD_FIXES, YAML_FIELD_ID, YAML_FIELD_LANGUAGE, YAML_FIELD_LINT_SELECT,
    YAML_FIELD_MESSAGE, YAML_FIELD_METRICS, YAML_FIELD_NAME, YAML_FIELD_NODE_TYPE,
    YAML_FIELD_PATTERN, YAML_FIELD_RATIONALE, YAML_FIELD_RULE, YAML_FIELD_SELECTORS,
    YAML_FIELD_SEVERITY, YAML_FIELD_TEMPLATE,
};
use crate::filters::rule_filters::RuleFilters;
use crate::utils::fs::collect_yaml_files;

/// Loaded and validated YAML rule
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    /// Unique identifier for the rule.
    pub id: String,
    /// Human-readable name of the rule.
    pub name: String,
    /// Category of the rule (e.g., quality, security).
    pub category: String,
    /// Severity level (error, warning, info).
    pub severity: String,
    /// Whether the rule is active.
    pub enabled: bool,
    /// Detailed description of what the rule checks.
    pub description: String,
    /// Explanation of why this rule exists.
    pub rationale: String,
    /// The engine used to execute this rule.
    pub engine: String,
    /// Engine-specific configuration.
    pub config: serde_json::Value,
    /// Raw rule definition.
    pub rule_definition: serde_json::Value,
    /// List of available automated fixes.
    pub fixes: Vec<RuleFix>,
    /// Linter codes to execute (e.g., `["F401"]` for Ruff, `["clippy::unwrap_used"]` for Clippy)
    pub lint_select: Vec<String>,
    /// Custom message for violations
    pub message: Option<String>,
    /// AST selectors for multi-language pattern matching (Phase 2)
    pub selectors: Vec<AstSelector>,
    /// Tree-sitter query string for complex AST matching (Phase 2)
    pub ast_query: Option<String>,
    /// Metrics configuration for schema v3 rules (Phase 4)
    pub metrics: Option<MetricsConfig>,
    /// Optional filters to restrict rule applicability by language, dependency, or file pattern.
    pub filters: Option<RuleFilters>,
}

/// Metrics configuration for rule/v3 rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Cognitive complexity threshold
    pub cognitive_complexity: Option<MetricThresholdConfig>,
    /// Cyclomatic complexity threshold
    pub cyclomatic_complexity: Option<MetricThresholdConfig>,
    /// Function length threshold
    pub function_length: Option<MetricThresholdConfig>,
    /// Nesting depth threshold
    pub nesting_depth: Option<MetricThresholdConfig>,
}

/// Configuration for a single metric threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholdConfig {
    /// Maximum allowed value
    pub max: u32,
    /// Severity level when threshold is exceeded
    pub severity: Option<String>,
    /// Languages this threshold applies to
    pub languages: Option<Vec<String>>,
}

/// AST selector for language-specific pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSelector {
    /// Programming language (e.g., "rust", "python")
    pub language: String,
    /// AST node type to match (e.g., "`call_expression`", "`function_definition`")
    pub node_type: String,
    /// Tree-sitter query pattern (optional, for complex matching)
    pub pattern: Option<String>,
}

/// Suggested fix for a rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFix {
    /// Type of fix (e.g., replacement, suppression).
    pub fix_type: String,
    /// Pattern to replace (if applicable).
    pub pattern: Option<String>,
    /// Message describing the fix.
    pub message: String,
}

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
                if path.ends_with(".yml") && !path.contains("/templates/") {
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

            for path in collect_yaml_files(&self.rules_dir)? {
                if Self::is_rule_file(&path) {
                    let content =
                        std::fs::read_to_string(&path).map_err(crate::ValidationError::Io)?;
                    let loaded_rules = self.load_rule_from_str(&path, &content)?;
                    rules.extend(loaded_rules);
                }
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

        // Apply template if specified
        let processed_yaml = if let Some(template_name) =
            yaml_value.get(YAML_FIELD_TEMPLATE).and_then(|v| v.as_str())
        {
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
                .apply_template(template_name, &template_args)?
        } else {
            yaml_value
        };

        // Handle extends
        let processed_yaml = if let Some(extends_name) = processed_yaml
            .get(YAML_FIELD_EXTENDS)
            .and_then(|v| v.as_str())
        {
            self.template_engine
                .extend_rule(extends_name, &processed_yaml)?
        } else {
            processed_yaml
        };

        // Substitute globals if provided
        let mut final_yaml = processed_yaml;
        if let Some(vars) = &self.variables {
            self.template_engine
                .substitute_variables(&mut final_yaml, vars)?;
        }

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

    /// Check if a file is a rule file
    fn is_rule_file(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("yml")
            && !path.to_str().is_some_and(|s| s.contains("/templates/"))
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

        let name = obj
            .get(YAML_FIELD_NAME)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_NAME)
            .to_owned();

        let category = obj
            .get(YAML_FIELD_CATEGORY)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_CATEGORY)
            .to_owned();

        let severity = obj
            .get(YAML_FIELD_SEVERITY)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_SEVERITY)
            .to_owned();

        let enabled = obj
            .get(YAML_FIELD_ENABLED)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        let description = obj
            .get(YAML_FIELD_DESCRIPTION)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_DESCRIPTION)
            .to_owned();

        let rationale = obj
            .get(YAML_FIELD_RATIONALE)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_RATIONALE)
            .to_owned();

        let engine = obj
            .get(YAML_FIELD_ENGINE)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_ENGINE)
            .to_owned();

        let config = obj
            .get(YAML_FIELD_CONFIG)
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let rule_definition = obj
            .get(YAML_FIELD_RULE)
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let fixes = obj
            .get(YAML_FIELD_FIXES)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|fix| {
                        if let Some(fix_obj) = fix.as_object() {
                            Some(RuleFix {
                                fix_type: fix_obj.get(YAML_FIELD_FIX_TYPE)?.as_str()?.to_owned(),
                                pattern: fix_obj
                                    .get(YAML_FIELD_PATTERN)
                                    .and_then(|v| v.as_str())
                                    .map(str::to_owned),
                                message: fix_obj.get(YAML_FIELD_MESSAGE)?.as_str()?.to_owned(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default();

        // Extract lint_select codes (for Ruff/Clippy integration)
        let lint_select = obj
            .get(YAML_FIELD_LINT_SELECT)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|code| code.as_str().map(str::to_owned))
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default();

        // Extract custom message
        let message = obj
            .get(YAML_FIELD_MESSAGE)
            .and_then(|v| v.as_str())
            .map(str::to_owned);

        // Extract AST selectors (Phase 2)
        let selectors = obj
            .get(YAML_FIELD_SELECTORS)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|sel| {
                        if let Some(sel_obj) = sel.as_object() {
                            Some(AstSelector {
                                language: sel_obj.get(YAML_FIELD_LANGUAGE)?.as_str()?.to_owned(),
                                node_type: sel_obj.get(YAML_FIELD_NODE_TYPE)?.as_str()?.to_owned(),
                                pattern: sel_obj
                                    .get(YAML_FIELD_PATTERN)
                                    .and_then(|v| v.as_str())
                                    .map(str::to_owned),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            // INTENTIONAL: YAML field extraction; missing field yields empty string
            .unwrap_or_default();

        // Extract ast_query (Phase 2)
        let ast_query = obj
            .get(YAML_FIELD_AST_QUERY)
            .and_then(|v| v.as_str())
            .map(str::to_owned);

        // Extract metrics configuration (Phase 4 - rule/v3)
        let metrics = obj
            .get(YAML_FIELD_METRICS)
            .and_then(|v| serde_json::from_value::<MetricsConfig>(v.clone()).ok());

        let filters = obj
            .get(YAML_FIELD_FILTERS)
            .and_then(|v| serde_json::from_value::<RuleFilters>(v.clone()).ok());

        Ok(ValidatedRule {
            id,
            name,
            category,
            severity,
            enabled,
            description,
            rationale,
            engine,
            config,
            rule_definition,
            fixes,
            lint_select,
            message,
            selectors,
            ast_query,
            metrics,
            filters,
        })
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
