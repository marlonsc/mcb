//! YAML Rule Validator
//!
//! Validates YAML rules against JSON Schema using jsonschema crate.

use jsonschema::Validator;
use serde_json::Value;
use std::path::Path;

use crate::Result;

/// Validator for YAML-based rules using JSON Schema
pub struct YamlRuleValidator {
    schema: Validator,
}

impl YamlRuleValidator {
    /// Create a new validator with the schema
    pub fn new() -> Result<Self> {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules/schema.json");

        let schema_content =
            std::fs::read_to_string(&schema_path).map_err(crate::ValidationError::Io)?;

        let schema_value: Value =
            serde_json::from_str(&schema_content).map_err(|e| crate::ValidationError::Parse {
                file: schema_path,
                message: format!("Schema parse error: {e}"),
            })?;

        let schema = jsonschema::validator_for(&schema_value).map_err(|e| {
            crate::ValidationError::Config(format!("Schema compilation error: {e:?}"))
        })?;

        Ok(Self { schema })
    }

    /// Validate a rule against the schema
    pub fn validate_rule(&self, rule: &Value) -> Result<()> {
        let errors: Vec<String> = self
            .schema
            .iter_errors(rule)
            .map(|e| format!("{}: {}", e.instance_path(), e))
            .collect();

        if !errors.is_empty() {
            return Err(crate::ValidationError::Config(format!(
                "Rule validation failed:\n{}",
                errors.join("\n")
            )));
        }

        Ok(())
    }

    /// Validate multiple rules
    pub fn validate_rules(&self, rules: &[Value]) -> Result<()> {
        for (index, rule) in rules.iter().enumerate() {
            if let Err(e) = self.validate_rule(rule) {
                return Err(crate::ValidationError::Config(format!(
                    "Rule at index {index} validation failed: {e}"
                )));
            }
        }
        Ok(())
    }

    /// Validate YAML against schema
    pub fn validate_yaml(&self, yaml_value: &serde_yaml::Value) -> Result<()> {
        let json_value =
            serde_json::to_value(yaml_value).map_err(|e| crate::ValidationError::Parse {
                file: "yaml_rule".into(),
                message: format!("JSON conversion error: {e}"),
            })?;

        let errors: Vec<String> = self
            .schema
            .iter_errors(&json_value)
            .map(|e| format!("{}: {}", e.instance_path(), e))
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::ValidationError::Config(format!(
                "Schema validation failed: {}",
                errors.join(", ")
            )))
        }
    }

    /// Create validator from custom schema
    pub fn from_schema(schema: &Value) -> Result<Self> {
        let compiled_schema = jsonschema::validator_for(schema).map_err(|e| {
            crate::ValidationError::Config(format!("Schema compilation error: {e:?}"))
        })?;

        Ok(Self {
            schema: compiled_schema,
        })
    }
}
