//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! YAML Rule Validator
//!
//! Validates YAML rules against JSON Schema using jsonschema crate.

use jsonschema::Validator;
use serde_json::Value;

use crate::Result;
use crate::embedded_rules::EmbeddedRules;

/// Validator for YAML-based rules using JSON Schema
pub struct YamlRuleValidator {
    schema: Validator,
}

impl YamlRuleValidator {
    /// Create a new validator with the schema
    ///
    /// # Errors
    ///
    /// Returns an error if the embedded JSON schema cannot be parsed or compiled.
    pub fn new() -> Result<Self> {
        let schema_content = EmbeddedRules::SCHEMA_JSON;

        let schema_value: Value =
            serde_json::from_str(schema_content).map_err(|e| crate::ValidationError::Parse {
                file: "embedded://rules/schema.json".into(),
                message: format!("Schema parse error: {e}"),
            })?;

        let schema = jsonschema::validator_for(&schema_value).map_err(|e| {
            crate::ValidationError::Config(format!("Schema compilation error: {e:?}"))
        })?;

        Ok(Self { schema })
    }

    /// Validate a rule against the schema
    ///
    /// # Errors
    ///
    /// Returns an error if the rule does not conform to the schema.
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
    ///
    /// # Errors
    ///
    /// Returns an error if any rule does not conform to the schema.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML value does not conform to the schema.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be compiled.
    pub fn from_schema(schema: &Value) -> Result<Self> {
        let compiled_schema = jsonschema::validator_for(schema).map_err(|e| {
            crate::ValidationError::Config(format!("Schema compilation error: {e:?}"))
        })?;

        Ok(Self {
            schema: compiled_schema,
        })
    }
}
