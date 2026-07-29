//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Pattern Registry Implementation
//!
//! Loads regex patterns from YAML rules and provides centralized access.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::Result;
use crate::rules::templates::TemplateEngine;
use mcb_domain::error;
use mcb_utils::constants::validate::CARGO_TOML_FILENAME;
use mcb_utils::constants::validate::{
    YAML_FIELD_ALLOWED_DEPS, YAML_FIELD_CONFIG, YAML_FIELD_CRATE_NAME, YAML_FIELD_ID,
    YAML_FIELD_PATTERNS, YAML_FIELD_REGEX, YAML_FIELD_SELECTORS,
};

/// Registry of compiled regex patterns and configurations loaded from YAML rules
pub struct PatternRegistry {
    patterns: HashMap<String, Regex>,
    configs: HashMap<String, serde_yaml::Value>,
}

impl PatternRegistry {
    /// Create an empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Load patterns from all YAML rules in a directory, using config for template variables
    ///
    /// # Errors
    ///
    /// Returns an error if rule file enumeration fails.
    pub fn load_from_rules(
        rules_dir: &Path,
        naming_config: &crate::config::NamingRulesConfig,
        project_prefix: &str,
    ) -> Result<Self> {
        let mut registry = Self::new();

        let rule_files = crate::utils::fs::collect_yaml_files(rules_dir)?;
        for path in rule_files.into_iter().filter(|p| !is_template_path(p)) {
            if let Err(e) = registry.load_rule_file(&path, naming_config, project_prefix) {
                mcb_domain::warn!(
                    "pattern_registry",
                    "Failed to load patterns/config",
                    &format!("path={} error={}", path.display(), e)
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
        let content = std::fs::read_to_string(path)?;
        let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let variables_value = template_variables(naming_config, project_prefix);
        let engine = TemplateEngine::new();
        if let Err(e) = engine.substitute_variables(&mut yaml, &variables_value) {
            mcb_domain::warn!(
                "pattern_registry",
                "Failed to substitute variables",
                &format!("path={} error={}", path.display(), e)
            );
        }

        // Get rule ID for namespacing
        let rule_id = yaml
            .get(YAML_FIELD_ID)
            .and_then(|v| v.as_str())
            .unwrap_or(mcb_utils::constants::FALLBACK_UNKNOWN)
            .to_owned();

        self.register_yaml_patterns(&yaml, &rule_id)?;

        // Load generic configuration from "config" section
        if let Some(config) = yaml.get(YAML_FIELD_CONFIG) {
            self.configs.insert(rule_id.clone(), config.clone());
        }

        self.merge_dependency_shorthand(&yaml, &rule_id);

        Ok(())
    }

    /// Register `patterns` and `selectors` sections of a rule file, namespaced by `rule_id`.
    fn register_yaml_patterns(&mut self, yaml: &serde_yaml::Value, rule_id: &str) -> Result<()> {
        for (name, pattern) in yaml
            .get(YAML_FIELD_PATTERNS)
            .and_then(|v| v.as_mapping())
            .into_iter()
            .flat_map(serde_yaml::Mapping::iter)
        {
            if let (Some(name_str), Some(pattern_str)) = (name.as_str(), pattern.as_str()) {
                let pattern_id = format!("{rule_id}.{name_str}");
                self.register_pattern(&pattern_id, pattern_str)?;
            }
        }

        for (i, selector) in yaml
            .get(YAML_FIELD_SELECTORS)
            .and_then(|v| v.as_sequence())
            .into_iter()
            .flat_map(|selectors| selectors.iter())
            .enumerate()
        {
            if let Some(pattern) = selector.get(YAML_FIELD_REGEX).and_then(|v| v.as_str()) {
                let pattern_id = format!("{rule_id}.selector_{i}");
                self.register_pattern(&pattern_id, pattern)?;
            }
        }

        Ok(())
    }

    /// Merge top-level `crate_name`/`allowed_dependencies` shorthand into the rule config.
    fn merge_dependency_shorthand(&mut self, yaml: &serde_yaml::Value, rule_id: &str) {
        let Some(crate_name) = yaml.get(YAML_FIELD_CRATE_NAME) else {
            return;
        };

        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::from(YAML_FIELD_CRATE_NAME),
            crate_name.clone(),
        );
        if let Some(allowed) = yaml.get(YAML_FIELD_ALLOWED_DEPS) {
            map.insert(
                serde_yaml::Value::from(YAML_FIELD_ALLOWED_DEPS),
                allowed.clone(),
            );
        }

        let entry = self
            .configs
            .entry(rule_id.to_owned())
            .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        if let Some(config_map) = entry.as_mapping_mut() {
            for (k, v) in map {
                config_map.insert(k, v);
            }
        }
    }

    /// Register a pattern with the given ID
    ///
    /// # Errors
    ///
    /// Returns an error if the regex pattern is invalid.
    pub fn register_pattern(&mut self, id: &str, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern).map_err(|e| {
            crate::ValidationError::Config(format!("Invalid regex pattern '{id}': {e}"))
        })?;
        self.patterns.insert(id.to_owned(), regex);
        Ok(())
    }

    /// Get a pattern by ID
    #[must_use]
    pub fn get(&self, pattern_id: &str) -> Option<&Regex> {
        self.patterns.get(pattern_id)
    }

    /// Get a configuration by rule ID
    #[must_use]
    pub fn get_config(&self, rule_id: &str) -> Option<&serde_yaml::Value> {
        self.configs.get(rule_id)
    }

    /// Get a list of strings from configuration
    #[must_use]
    pub fn get_config_list(&self, rule_id: &str, key: &str) -> Vec<String> {
        self.get_config(rule_id)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            // INTENTIONAL: YAML sequence parsing; empty patterns list is valid
            .unwrap_or_default()
    }

    /// Check if a pattern exists
    #[must_use]
    pub fn contains(&self, pattern_id: &str) -> bool {
        self.patterns.contains_key(pattern_id)
    }

    /// Get all pattern IDs
    pub fn pattern_ids(&self) -> impl Iterator<Item = &String> {
        self.patterns.keys()
    }

    /// Get the number of registered patterns
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if the registry is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

/// Build the YAML template variables (`*_crate`, `*_module`, `project_prefix`) from naming config.
fn template_variables(
    naming_config: &crate::config::NamingRulesConfig,
    project_prefix: &str,
) -> serde_yaml::Value {
    let mut variables = serde_yaml::Mapping::new();
    variables.insert(
        serde_yaml::Value::String("project_prefix".to_owned()),
        serde_yaml::Value::String(project_prefix.to_owned()),
    );

    let crates: [(&str, &str); 7] = [
        ("domain", &naming_config.domain_crate),
        ("application", &naming_config.application_crate),
        ("providers", &naming_config.providers_crate),
        ("infrastructure", &naming_config.infrastructure_crate),
        ("server", &naming_config.server_crate),
        ("validate", &naming_config.validate_crate),
        ("utils", &naming_config.utils_crate),
    ];

    for (key, crate_name) in crates {
        let module_name = crate_name.replace('-', "_");
        variables.insert(
            serde_yaml::Value::String(format!("{key}_crate")),
            serde_yaml::Value::String(crate_name.to_owned()),
        );
        variables.insert(
            serde_yaml::Value::String(format!("{key}_module")),
            serde_yaml::Value::String(module_name),
        );
    }

    serde_yaml::Value::Mapping(variables)
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

/// Get the default rules directory.
///
/// Resolution order (all workspace-relative unless overridden via env):
/// 1. `MCB_RULES_DIR` environment variable (explicit override)
/// 2. `CARGO_MANIFEST_DIR/rules` (building mcb-validate directly)
/// 3. Workspace root `crates/mcb-validate/rules` (used as dependency)
/// 4. CWD-relative `crates/mcb-validate/rules` (running from workspace root)
/// 5. CWD-relative `rules/` fallback
#[must_use]
pub fn default_rules_dir() -> PathBuf {
    // 1. Explicit override via environment variable
    if let Some(dir) = env_rules_dir() {
        return dir;
    }

    // 2-3. Derived from CARGO_MANIFEST_DIR (direct build or workspace dependency)
    if let Some(dir) = manifest_rules_dir() {
        return dir;
    }

    // 4. Try relative to current directory (works when running from workspace root)
    let cwd_rules = PathBuf::from("crates/mcb-validate/rules");
    if cwd_rules.exists() {
        return cwd_rules;
    }

    // 5. Fallback to CWD-relative rules/
    PathBuf::from("rules")
}

/// Rules directory from the `MCB_RULES_DIR` override, if it exists.
fn env_rules_dir() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var("MCB_RULES_DIR").ok()?);
    path.exists().then_some(path)
}

/// Rules directory derived from `CARGO_MANIFEST_DIR` (direct build or workspace dependency).
fn manifest_rules_dir() -> Option<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;

    let rules_dir = PathBuf::from(&manifest_dir).join("rules");
    if rules_dir.exists() {
        return Some(rules_dir);
    }

    let workspace_root = PathBuf::from(&manifest_dir)
        .ancestors()
        .find(|p| p.join(CARGO_TOML_FILENAME).exists() && p.join("crates").exists())?
        .to_path_buf();
    let validate_rules = workspace_root.join("crates/mcb-validate/rules");
    validate_rules.exists().then_some(validate_rules)
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
            error!("pattern_registry", "Failed to load pattern registry", &e);
            PatternRegistry::new()
        },
    )
});
