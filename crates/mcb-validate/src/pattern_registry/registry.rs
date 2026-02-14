//! Pattern Registry Implementation
//!
//! Loads regex patterns from YAML rules and provides centralized access.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;
use tracing::{error, warn};

use crate::Result;
use crate::rules::templates::TemplateEngine;

/// Registry of compiled regex patterns and configurations loaded from YAML rules
pub struct PatternRegistry {
    patterns: HashMap<String, Regex>,
    configs: HashMap<String, serde_yaml::Value>,
}

impl PatternRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Load patterns from all YAML rules in a directory, using config for template variables
    pub fn load_from_rules(
        rules_dir: &Path,
        naming_config: &crate::config::NamingRulesConfig,
        project_prefix: &str,
    ) -> Result<Self> {
        let mut registry = Self::new();

        for path in collect_rule_files(rules_dir) {
            if let Err(e) = registry.load_rule_file(&path, naming_config, project_prefix) {
                warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to load patterns/config"
                );
            }
        }

        Ok(registry)
    }

    /// Load patterns and configurations from a single YAML rule file
    fn load_rule_file(
        &mut self,
        path: &Path,
        naming_config: &crate::config::NamingRulesConfig,
        project_prefix: &str,
    ) -> Result<()> {
        if is_template_path(path) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Build template variables from configuration (no hardcoded crate names)
        let mut variables = serde_yaml::Mapping::new();
        variables.insert(
            serde_yaml::Value::String("project_prefix".to_string()),
            serde_yaml::Value::String(project_prefix.to_string()),
        );

        // Map each layer key to (crate_name, module_name) from NamingRulesConfig
        let crates: [(&str, &str); 6] = [
            ("domain", &naming_config.domain_crate),
            ("application", &naming_config.application_crate),
            ("providers", &naming_config.providers_crate),
            ("infrastructure", &naming_config.infrastructure_crate),
            ("server", &naming_config.server_crate),
            ("validate", &naming_config.validate_crate),
        ];

        for (key, crate_name) in crates {
            let module_name = crate_name.replace('-', "_");
            variables.insert(
                serde_yaml::Value::String(format!("{key}_crate")),
                serde_yaml::Value::String(crate_name.to_string()),
            );
            variables.insert(
                serde_yaml::Value::String(format!("{key}_module")),
                serde_yaml::Value::String(module_name),
            );
        }

        let engine = TemplateEngine::new();
        let variables_value = serde_yaml::Value::Mapping(variables);
        if let Err(e) = engine.substitute_variables(&mut yaml, &variables_value) {
            warn!(
                path = %path.display(),
                error = %e,
                "Failed to substitute variables"
            );
        }

        // Get rule ID for namespacing
        let rule_id = yaml.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

        // Load patterns from "patterns" section
        if let Some(patterns) = yaml.get("patterns").and_then(|v| v.as_mapping()) {
            for (name, pattern) in patterns {
                if let (Some(name_str), Some(pattern_str)) = (name.as_str(), pattern.as_str()) {
                    let pattern_id = format!("{rule_id}.{name_str}");
                    self.register_pattern(&pattern_id, pattern_str)?;
                }
            }
        }

        // Load patterns from "selectors" section (for AST patterns)
        if let Some(selectors) = yaml.get("selectors").and_then(|v| v.as_sequence()) {
            for (i, selector) in selectors.iter().enumerate() {
                if let Some(pattern) = selector.get("regex").and_then(|v| v.as_str()) {
                    let pattern_id = format!("{rule_id}.selector_{i}");
                    self.register_pattern(&pattern_id, pattern)?;
                }
            }
        }

        // Load generic configuration from "config" section
        if let Some(config) = yaml.get("config") {
            self.configs.insert(rule_id.to_string(), config.clone());
        }

        // Also load top-level crate_name and allowed_dependencies if present (shorthand for dependency rules)
        if let Some(crate_name) = yaml.get("crate_name") {
            let mut map = serde_yaml::Mapping::new();
            map.insert(serde_yaml::Value::from("crate_name"), crate_name.clone());
            if let Some(allowed) = yaml.get("allowed_dependencies") {
                map.insert(
                    serde_yaml::Value::from("allowed_dependencies"),
                    allowed.clone(),
                );
            }

            // Merge into config for this rule if it doesn't already have one, or extend it
            let entry = self
                .configs
                .entry(rule_id.to_string())
                .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
            if let Some(config_map) = entry.as_mapping_mut() {
                for (k, v) in map {
                    config_map.insert(k, v);
                }
            }
        }

        Ok(())
    }

    /// Register a pattern with the given ID
    pub fn register_pattern(&mut self, id: &str, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern).map_err(|e| {
            crate::ValidationError::Config(format!("Invalid regex pattern '{id}': {e}"))
        })?;
        self.patterns.insert(id.to_string(), regex);
        Ok(())
    }

    /// Get a pattern by ID
    pub fn get(&self, pattern_id: &str) -> Option<&Regex> {
        self.patterns.get(pattern_id)
    }

    /// Get a configuration by rule ID
    pub fn get_config(&self, rule_id: &str) -> Option<&serde_yaml::Value> {
        self.configs.get(rule_id)
    }

    /// Get a list of strings from configuration
    pub fn get_config_list(&self, rule_id: &str, key: &str) -> Vec<String> {
        self.get_config(rule_id)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a pattern exists
    pub fn contains(&self, pattern_id: &str) -> bool {
        self.patterns.contains_key(pattern_id)
    }

    /// Get all pattern IDs
    pub fn pattern_ids(&self) -> impl Iterator<Item = &String> {
        self.patterns.keys()
    }

    /// Get the number of registered patterns
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

fn is_template_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "templates")
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_rule_files(rules_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![rules_dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext == "yml" || ext == "yaml")
                && !is_template_path(&path)
            {
                files.push(path);
            }
        }
    }

    files
}

/// Get the default rules directory
pub fn default_rules_dir() -> PathBuf {
    // 1. Try CARGO_MANIFEST_DIR (works when building mcb-validate directly)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let rules_dir = PathBuf::from(&manifest_dir).join("rules");
        if rules_dir.exists() {
            return rules_dir;
        }

        // 2. When used as dependency, CARGO_MANIFEST_DIR points to consumer crate
        // Try to find mcb-validate/rules relative to workspace root
        if let Some(workspace_root) = PathBuf::from(&manifest_dir)
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists() && p.join("crates").exists())
        {
            let validate_rules = workspace_root.join("crates/mcb-validate/rules");
            if validate_rules.exists() {
                return validate_rules;
            }
        }
    }

    // 3. Try relative to current directory (works when running from workspace root)
    let cwd_rules = PathBuf::from("crates/mcb-validate/rules");
    if cwd_rules.exists() {
        return cwd_rules;
    }

    // 4. Check ~/.local/share/mcb/rules (make install target)
    if let Some(home) = std::env::var_os("HOME") {
        let xdg_rules = PathBuf::from(home).join(".local/share/mcb/rules");
        if xdg_rules.exists() {
            return xdg_rules;
        }
    }

    // 5. Try /usr/share/mcb/rules (system-wide)
    let system_rules = PathBuf::from("/usr/share/mcb/rules");
    if system_rules.exists() {
        return system_rules;
    }

    // 6. Fallback
    PathBuf::from("rules")
}

/// Global pattern registry, lazy-loaded from YAML rules and configuration
pub static PATTERNS: std::sync::LazyLock<PatternRegistry> = std::sync::LazyLock::new(|| {
    let rules_dir = default_rules_dir();
    // Load config to get project-specific crate names for template substitution
    let file_config = crate::config::FileConfig::load(".");
    let naming_config = &file_config.rules.naming;
    let project_prefix = &file_config.general.project_prefix;
    PatternRegistry::load_from_rules(&rules_dir, naming_config, project_prefix).unwrap_or_else(
        |e| {
            error!(error = %e, "Failed to load pattern registry");
            PatternRegistry::new()
        },
    )
});
