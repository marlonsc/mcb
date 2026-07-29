//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
//! Embedded default validation rules for self-contained execution.

include!(concat!(env!("OUT_DIR"), "/embedded_rules_gen.rs"));

/// Embedded YAML defaults for mcb-validate.
pub struct EmbeddedRules;

impl EmbeddedRules {
    /// Embedded JSON schema used to validate YAML rule structure.
    pub const SCHEMA_JSON: &'static str = include_str!("../rules/schema.json");

    /// All embedded YAML files including templates.
    #[must_use]
    pub fn all_yaml() -> Vec<(&'static str, &'static str)> {
        EMBEDDED_RULES.to_vec()
    }

    /// Embedded YAML rule files excluding template definitions.
    #[must_use]
    pub fn rule_yaml() -> Vec<(&'static str, &'static str)> {
        Self::all_yaml()
            .into_iter()
            .filter(|(path, _)| !path.contains("/templates/") && !path.starts_with("templates/"))
            .collect()
    }
}
