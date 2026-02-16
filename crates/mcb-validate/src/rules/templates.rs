//! Template Engine for YAML Rules
//!
//! Provides template inheritance and variable substitution for DRY rule definitions.

use std::collections::HashMap;
use std::path::Path;

use serde_yaml;

use crate::Result;
use crate::utils::fs::collect_yaml_files;

/// Template engine for YAML rules with inheritance and substitution
pub struct TemplateEngine {
    templates: HashMap<String, serde_yaml::Value>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    /// Create a new template engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Load all templates from the templates directory
    ///
    /// # Errors
    ///
    /// Returns an error if template files cannot be read or parsed.
    pub async fn load_templates(&mut self, rules_dir: &Path) -> Result<()> {
        let templates_dir = rules_dir.join("templates");

        if !templates_dir.exists() {
            return Ok(()); // No templates directory, that's fine
        }

        for path in collect_yaml_files(&templates_dir)? {
            let path = path.as_path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                let template_name =
                    path.file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            crate::ValidationError::Config(format!(
                                "Invalid template filename: {}",
                                path.display()
                            ))
                        })?;

                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(crate::ValidationError::Io)?;

                self.parse_and_add_template(path, &content, template_name)?;
            }
        }

        Ok(())
    }

    /// Load all templates from the templates directory (Synchronous version)
    ///
    /// # Errors
    ///
    /// Returns an error if template files cannot be read or parsed.
    pub fn load_templates_sync(&mut self, rules_dir: &Path) -> Result<()> {
        let templates_dir = rules_dir.join("templates");

        if !templates_dir.exists() {
            return Ok(());
        }

        for path in collect_yaml_files(&templates_dir)? {
            let path = path.as_path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                let template_name =
                    path.file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            crate::ValidationError::Config(format!(
                                "Invalid template filename: {}",
                                path.display()
                            ))
                        })?;

                let content = std::fs::read_to_string(path).map_err(crate::ValidationError::Io)?;

                self.parse_and_add_template(path, &content, template_name)?;
            }
        }

        Ok(())
    }

    fn parse_and_add_template(
        &mut self,
        path: &Path,
        content: &str,
        template_name: &str,
    ) -> Result<()> {
        let template: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("Template parse error: {e}"),
            })?;

        // Verify this is actually a template
        if template
            .get("_base")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false)
        {
            // Use the 'name' field from template YAML if present, otherwise use filename
            let registry_name = template.get("name").and_then(|v| v.as_str()).map_or_else(
                || template_name.to_owned(),
                std::string::ToString::to_string,
            );
            self.templates.insert(registry_name, template);
        }
        Ok(())
    }

    /// Load templates from embedded `(path, content)` entries.
    ///
    /// # Errors
    ///
    /// Returns an error if template parsing fails.
    pub fn load_templates_from_embedded(&mut self, entries: &[(String, String)]) -> Result<()> {
        for (path, content) in entries {
            if !path.ends_with(".yml") || !path.contains("/templates/") {
                continue;
            }

            let template_name = Path::new(path)
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    crate::ValidationError::Config(format!("Invalid template filename: {path}"))
                })?;

            let template: serde_yaml::Value =
                serde_yaml::from_str(content).map_err(|e| crate::ValidationError::Parse {
                    file: path.into(),
                    message: format!("Template parse error: {e}"),
                })?;

            if template
                .get("_base")
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false)
            {
                let registry_name = template.get("name").and_then(|v| v.as_str()).map_or_else(
                    || template_name.to_owned(),
                    std::string::ToString::to_string,
                );
                self.templates.insert(registry_name, template);
            }
        }

        Ok(())
    }

    /// Apply a template to a rule definition
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or variable substitution fails.
    pub fn apply_template(
        &self,
        template_name: &str,
        rule: &serde_yaml::Value,
    ) -> Result<serde_yaml::Value> {
        let template = self.templates.get(template_name).ok_or_else(|| {
            crate::ValidationError::Config(format!("Template '{template_name}' not found"))
        })?;

        // Start with the template as base
        let mut result = template.clone();

        // Override with rule-specific values
        Self::merge_yaml_values(&mut result, rule);

        // Process variable substitutions
        self.substitute_variables(&mut result, rule)?;

        // Remove template metadata (but keep name if rule provided one)
        if let Some(obj) = result.as_mapping_mut() {
            obj.remove("_base");
            // Only remove template's internal name if rule didn't provide its own
            if rule.get("name").is_none() {
                obj.remove("name");
            }
        }

        Ok(result)
    }

    /// Extend a rule with another rule (inheritance)
    ///
    /// # Errors
    ///
    /// Returns an error if the base rule cannot be resolved.
    pub fn extend_rule(
        &self,
        _extends_name: &str,
        rule: &serde_yaml::Value,
    ) -> Result<serde_yaml::Value> {
        // For now, just return the rule as-is
        // In a full implementation, this would look up the base rule
        // and merge it with the extending rule
        Ok(rule.clone())
    }

    /// Merge two YAML values (rule overrides template)
    fn merge_yaml_values(base: &mut serde_yaml::Value, override_value: &serde_yaml::Value) {
        if let (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(override_map)) =
            (base, override_value)
        {
            for (key, override_val) in override_map {
                base_map.insert(key.clone(), override_val.clone());
            }
        }
    }

    /// Substitute variables in the form {{`variable_name`}}
    ///
    /// # Errors
    ///
    /// Returns an error if a referenced variable is not found.
    pub fn substitute_variables(
        &self,
        value: &mut serde_yaml::Value,
        variables: &serde_yaml::Value,
    ) -> Result<()> {
        match value {
            serde_yaml::Value::String(s) => {
                *s = Self::substitute_string(s, variables)?;
            }
            serde_yaml::Value::Mapping(map) => {
                for val in map.values_mut() {
                    self.substitute_variables(val, variables)?;
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    self.substitute_variables(item, variables)?;
                }
            }
            serde_yaml::Value::Null
            | serde_yaml::Value::Bool(_)
            | serde_yaml::Value::Number(_)
            | serde_yaml::Value::Tagged(_) => {}
        }
        Ok(())
    }

    /// Substitute variables in a string
    fn substitute_string(input: &str, variables: &serde_yaml::Value) -> Result<String> {
        let mut result = input.to_owned();

        // Find all {{variable}} patterns
        // Find all {{variable}} patterns, allowing for spaces
        let var_pattern = regex::Regex::new(r"\{\{\s*(\w+)\s*\}\}")
            .map_err(|e| crate::ValidationError::Config(format!("Regex error: {e}")))?;

        for capture in var_pattern.captures_iter(input) {
            if let Some(var_name) = capture.get(1) {
                let Some(full_match) = capture.get(0) else {
                    continue;
                };
                let var_value = Self::get_variable_value(variables, var_name.as_str())?;
                result = result.replace(full_match.as_str(), &var_value);
            }
        }

        Ok(result)
    }

    /// Get variable value from the variables YAML
    fn get_variable_value(variables: &serde_yaml::Value, var_name: &str) -> Result<String> {
        if let Some(value) = variables.get(var_name) {
            match value {
                serde_yaml::Value::String(s) => Ok(s.clone()),
                serde_yaml::Value::Number(n) => Ok(n.to_string()),
                serde_yaml::Value::Bool(b) => Ok(b.to_string()),
                serde_yaml::Value::Sequence(seq) => {
                    // For arrays, join with commas (for patterns)
                    let strings: Vec<String> = seq
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(std::string::ToString::to_string)
                        .collect();
                    Ok(strings.join(","))
                }
                serde_yaml::Value::Null
                | serde_yaml::Value::Mapping(_)
                | serde_yaml::Value::Tagged(_) => Ok(format!("{value:?}")),
            }
        } else {
            Err(crate::ValidationError::Config(format!(
                "Variable '{var_name}' not found"
            )))
        }
    }

    /// Get available templates
    #[must_use]
    pub fn get_templates(&self) -> &HashMap<String, serde_yaml::Value> {
        &self.templates
    }

    /// Check if a template exists
    #[must_use]
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}
