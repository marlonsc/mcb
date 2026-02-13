//! YAML Rule Loader
//!
//! Automatically loads and validates YAML-based rules with template support.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml;

use super::templates::TemplateEngine;
use super::yaml_validator::YamlRuleValidator;
use crate::Result;

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
    pub fn new(rules_dir: PathBuf) -> Result<Self> {
        Self::with_variables(rules_dir, None)
    }

    /// Create a new YAML rule loader with variables
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
    pub fn from_embedded(rules: &[(&str, &str)]) -> Result<Self> {
        Self::from_embedded_with_variables(rules, None)
    }

    /// Create a YAML loader backed by embedded entries with substitution variables.
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
                    .map(|(path, content)| ((*path).to_string(), (*content).to_string()))
                    .collect(),
            ),
        })
    }

    /// Load all rules from embedded entries without filesystem access.
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

    /// Load all rules from the rules directory
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
            if self.is_rule_file(&path) {
                let loaded_rules = self.load_rule_file(&path).await?;
                rules.extend(loaded_rules);
            }
        }

        Ok(rules)
    }

    /// Load rules from a specific file
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
            .get("_base")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false)
        {
            // This is a template, skip it
            return Ok(vec![]);
        }

        // Apply template if specified
        let processed_yaml =
            if let Some(template_name) = yaml_value.get("_template").and_then(|v| v.as_str()) {
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
        let processed_yaml =
            if let Some(extends_name) = processed_yaml.get("_extends").and_then(|v| v.as_str()) {
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
        let validated_rule = self.yaml_to_validated_rule(&json_value)?;

        Ok(vec![validated_rule])
    }

    /// Check if a file is a rule file
    fn is_rule_file(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("yml")
            && !path.to_string_lossy().contains("/templates/")
    }

    /// Convert YAML/JSON value to `ValidatedRule`
    #[allow(clippy::too_many_lines)]
    fn yaml_to_validated_rule(&self, value: &serde_json::Value) -> Result<ValidatedRule> {
        let obj = value
            .as_object()
            .ok_or_else(|| crate::ValidationError::Config("Rule must be an object".to_string()))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config("Rule must have an 'id' field".to_string())
            })?
            .to_string();

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Rule")
            .to_string();

        let category = obj
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("quality")
            .to_string();

        let severity = obj
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("warning")
            .to_string();

        let enabled = obj
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        let rationale = obj
            .get("rationale")
            .and_then(|v| v.as_str())
            .unwrap_or("No rationale provided")
            .to_string();

        let engine = obj
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("rusty-rules")
            .to_string();

        let config = obj
            .get("config")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let rule_definition = obj
            .get("rule")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let fixes = obj
            .get("fixes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|fix| {
                        if let Some(fix_obj) = fix.as_object() {
                            Some(RuleFix {
                                fix_type: fix_obj.get("type")?.as_str()?.to_string(),
                                pattern: fix_obj
                                    .get("pattern")
                                    .and_then(|v| v.as_str())
                                    .map(std::string::ToString::to_string),
                                message: fix_obj.get("message")?.as_str()?.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract lint_select codes (for Ruff/Clippy integration)
        let lint_select = obj
            .get("lint_select")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|code| code.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        // Extract custom message
        let message = obj
            .get("message")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        // Extract AST selectors (Phase 2)
        let selectors = obj
            .get("selectors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|sel| {
                        if let Some(sel_obj) = sel.as_object() {
                            Some(AstSelector {
                                language: sel_obj.get("language")?.as_str()?.to_string(),
                                node_type: sel_obj.get("node_type")?.as_str()?.to_string(),
                                pattern: sel_obj
                                    .get("pattern")
                                    .and_then(|v| v.as_str())
                                    .map(std::string::ToString::to_string),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract ast_query (Phase 2)
        let ast_query = obj
            .get("ast_query")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        // Extract metrics configuration (Phase 4 - rule/v3)
        let metrics = obj
            .get("metrics")
            .and_then(|v| serde_json::from_value::<MetricsConfig>(v.clone()).ok());

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
        })
    }

    /// Get rule file path for a rule ID
    pub fn get_rule_path(&self, rule_id: &str) -> Option<PathBuf> {
        // This would need a more sophisticated mapping
        // For now, just search in the rules directory
        for path in collect_yaml_files(&self.rules_dir).ok()? {
            if self.is_rule_file(&path)
                && let Ok(content) = std::fs::read_to_string(&path)
                && content.contains(&format!("id: {rule_id}"))
            {
                return Some(path.to_path_buf());
            }
        }
        None
    }
}

fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(crate::ValidationError::Io)? {
            let entry = entry.map_err(crate::ValidationError::Io)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(crate::ValidationError::Io)?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                files.push(path);
            }
        }
    }

    Ok(files)
}
