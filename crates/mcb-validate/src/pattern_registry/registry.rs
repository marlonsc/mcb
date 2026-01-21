//! Pattern Registry Implementation
//!
//! Loads regex patterns from YAML rules and provides centralized access.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::Result;

/// Registry of compiled regex patterns loaded from YAML rules
pub struct PatternRegistry {
    patterns: HashMap<String, Regex>,
}

impl PatternRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Load patterns from all YAML rules in a directory
    pub fn load_from_rules(rules_dir: &Path) -> Result<Self> {
        let mut registry = Self::new();

        for entry in WalkDir::new(rules_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yml" || ext == "yaml")
            })
        {
            if let Err(e) = registry.load_rule_file(entry.path()) {
                eprintln!(
                    "Warning: Failed to load patterns from {:?}: {}",
                    entry.path(),
                    e
                );
            }
        }

        Ok(registry)
    }

    /// Load patterns from a single YAML rule file
    fn load_rule_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Get rule ID for namespacing
        let rule_id = yaml.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

        // Load patterns from "patterns" section
        if let Some(patterns) = yaml.get("patterns").and_then(|v| v.as_mapping()) {
            for (name, pattern) in patterns {
                if let (Some(name_str), Some(pattern_str)) = (name.as_str(), pattern.as_str()) {
                    let pattern_id = format!("{}.{}", rule_id, name_str);
                    self.register_pattern(&pattern_id, pattern_str)?;
                }
            }
        }

        // Load patterns from "selectors" section (for AST patterns)
        if let Some(selectors) = yaml.get("selectors").and_then(|v| v.as_sequence()) {
            for (i, selector) in selectors.iter().enumerate() {
                if let Some(pattern) = selector.get("regex").and_then(|v| v.as_str()) {
                    let pattern_id = format!("{}.selector_{}", rule_id, i);
                    self.register_pattern(&pattern_id, pattern)?;
                }
            }
        }

        Ok(())
    }

    /// Register a pattern with the given ID
    pub fn register_pattern(&mut self, id: &str, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern).map_err(|e| {
            crate::ValidationError::Config(format!("Invalid regex pattern '{}': {}", id, e))
        })?;
        self.patterns.insert(id.to_string(), regex);
        Ok(())
    }

    /// Get a pattern by ID
    pub fn get(&self, pattern_id: &str) -> Option<&Regex> {
        self.patterns.get(pattern_id)
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

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default rules directory
pub fn default_rules_dir() -> PathBuf {
    // Try to find rules relative to the crate
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let rules_dir = manifest_dir.join("rules");
    if rules_dir.exists() {
        return rules_dir;
    }

    // Try relative to current directory
    let cwd_rules = PathBuf::from("crates/mcb-validate/rules");
    if cwd_rules.exists() {
        return cwd_rules;
    }

    // Fallback
    PathBuf::from("rules")
}

/// Global pattern registry, lazy-loaded from YAML rules
pub static PATTERNS: Lazy<PatternRegistry> = Lazy::new(|| {
    let rules_dir = default_rules_dir();
    PatternRegistry::load_from_rules(&rules_dir).unwrap_or_else(|e| {
        eprintln!("Error: Failed to load pattern registry: {}", e);
        PatternRegistry::new()
    })
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = PatternRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_pattern() {
        let mut registry = PatternRegistry::new();
        registry
            .register_pattern("test.pattern", r"\w+")
            .expect("Should register");

        assert!(registry.contains("test.pattern"));
        assert!(registry.get("test.pattern").is_some());
    }

    #[test]
    fn test_invalid_pattern() {
        let mut registry = PatternRegistry::new();
        let result = registry.register_pattern("test.invalid", r"[invalid");

        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_matching() {
        let mut registry = PatternRegistry::new();
        registry
            .register_pattern("test.fn_decl", r"fn\s+(\w+)")
            .expect("Should register");

        let pattern = registry.get("test.fn_decl").expect("Should exist");
        assert!(pattern.is_match("fn test_function()"));
        assert!(!pattern.is_match("let x = 1"));
    }

    #[test]
    fn test_load_from_rules_dir() {
        let rules_dir = default_rules_dir();
        if rules_dir.exists() {
            let registry = PatternRegistry::load_from_rules(&rules_dir).expect("Should load rules");
            // May or may not have patterns depending on YAML content
            println!("Loaded {} patterns from rules", registry.len());
        }
    }
}
