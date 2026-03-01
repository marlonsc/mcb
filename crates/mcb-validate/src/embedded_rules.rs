//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
//! Embedded default validation rules for self-contained execution.

/// Produce a `(&str, &str)` tuple from a relative rule path.
macro_rules! rule {
    ($path:literal) => {
        (
            concat!("rules/", $path),
            include_str!(concat!("../rules/", $path)),
        )
    };
}

/// Embedded YAML defaults for mcb-validate.
pub struct EmbeddedRules;

impl EmbeddedRules {
    /// Embedded JSON schema used to validate YAML rule structure.
    pub const SCHEMA_JSON: &'static str = include_str!("../rules/schema.json");

    /// All embedded YAML files including templates.
    #[must_use]
    pub fn all_yaml() -> Vec<(&'static str, &'static str)> {
        vec![
            rule!("clean-architecture/CA001_domain-independence.yml"),
            rule!("clean-architecture/CA002_application-boundaries.yml"),
            rule!("clean-architecture/CA003_domain-traits-only.yml"),
            rule!("clean-architecture/CA004_handler-dependency-injection.yml"),
            rule!("clean-architecture/CA005_entity-identity.yml"),
            rule!("clean-architecture/CA006_value-object-immutability.yml"),
            rule!("clean-architecture/CA007_infrastructure-concrete-imports.yml"),
            rule!("clean-architecture/CA008_application-port-imports.yml"),
            rule!("clean-architecture/CA009_infrastructure-no-application.yml"),
            rule!("clean-architecture/CA010_providers-boundaries.yml"),
            rule!("clean-architecture/CA011_infrastructure-boundaries.yml"),
            rule!("clean-architecture/CA012_server-boundaries.yml"),
            rule!("clean-architecture/CA013_mcb-boundaries.yml"),
            rule!("clean-architecture/CA014_validate-boundaries.yml"),
            rule!("clean-architecture/CA017_di-patterns.yml"),
            rule!("clean-architecture/LAYER001_layer-flow-rules.yml"),
            rule!("clean-architecture/PORT001_port-adapter-config.yml"),
            rule!("di/SHAKU001_component-derive.yml"),
            rule!("di/SHAKU002_interface-annotation.yml"),
            rule!("di/SHAKU003_concrete-type-handler.yml"),
            rule!("documentation/DOC001_completeness.yml"),
            rule!("duplication/DUP001_exact-clone.yml"),
            rule!("duplication/DUP002_renamed-clone.yml"),
            rule!("duplication/DUP003_gapped-clone.yml"),
            rule!("implementation/IMPL001_quality.yml"),
            rule!("metrics/METRIC001_cognitive-complexity.yml"),
            rule!("metrics/METRIC002_function-length.yml"),
            rule!("metrics/METRIC003_nesting-depth.yml"),
            rule!("metrics/METRIC004_cyclomatic-complexity.yml"),
            rule!("metrics/METRIC005_halstead-volume.yml"),
            rule!("metrics/METRIC006_maintainability-index.yml"),
            rule!("migration/CTOR001_shaku-migration.yml"),
            rule!("migration/CTOR002_constructor-injection.yml"),
            rule!("migration/CTOR003_manual-service-composition.yml"),
            rule!("migration/LINKME001_inventory-migration.yml"),
            rule!("migration/LINKME002_linkme-slice-declaration.yml"),
            rule!("migration/LINKME003_linkme-slice-usage.yml"),
            rule!("migration/ROCKET001_rocket-migration.yml"),
            rule!("migration/ROCKET002_rocket-attribute-handlers.yml"),
            rule!("migration/ROCKET003_rocket-route-organization.yml"),
            rule!("organization/ORG001_structure.yml"),
            rule!("organization/ORG015_adapter-location.yml"),
            rule!("organization/ORG017_handler-location.yml"),
            rule!("organization/ORG018_port-location.yml"),
            rule!("organization/ORG019_trait-placement.yml"),
            rule!("organization/ORG020_domain-adapters.yml"),
            rule!("organization/ORG021_infra-ports.yml"),
            rule!("organization/ORG022_scattered-config.yml"),
            rule!("organization/ORG023_nested-errors.yml"),
            rule!("organization/VIS001_visibility-config.yml"),
            rule!("performance/ASYNC001_patterns.yml"),
            rule!("quality/QUAL001_no-unwrap.yml"),
            rule!("quality/QUAL002_no-expect.yml"),
            rule!("quality/QUAL004_function-size-limit.yml"),
            rule!("quality/QUAL005_ruff-imports.yml"),
            rule!("quality/QUAL006_file-size-limit.yml"),
            rule!("refactoring/REF001_refactoring-config.yml"),
            rule!("solid/SOLID001_trait-methods.yml"),
            rule!("solid/SOLID002_impl-methods.yml"),
            rule!("solid/SOLID003_match-complexity.yml"),
            rule!("templates/cargo-dependency-template.yml"),
            rule!("templates/code-pattern-template.yml"),
            rule!("templates/import-check-template.yml"),
            rule!("testing/TEST001_organization.yml"),
        ]
    }

    /// Embedded YAML rule files excluding template definitions.
    #[must_use]
    pub fn rule_yaml() -> Vec<(&'static str, &'static str)> {
        Self::all_yaml()
            .into_iter()
            .filter(|(path, _)| !path.contains("/templates/"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::EmbeddedRules;

    #[test]
    fn embedded_yaml_entries_are_non_empty() {
        for (path, content) in EmbeddedRules::all_yaml() {
            assert!(
                !content.trim().is_empty(),
                "embedded rule '{path}' must not be empty"
            );
        }
    }
}
