//! Embedded default validation rules for self-contained execution.

/// Embedded YAML defaults for mcb-validate.
pub struct EmbeddedRules;

impl EmbeddedRules {
    /// Embedded JSON schema used to validate YAML rule structure.
    pub const SCHEMA_JSON: &'static str = include_str!("../rules/schema.json");

    const RULES_CLEAN_ARCHITECTURE_CA001_DOMAIN_INDEPENDENCE_YML: &'static str =
        include_str!("../rules/clean-architecture/CA001_domain-independence.yml");
    const RULES_CLEAN_ARCHITECTURE_CA002_APPLICATION_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA002_application-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA003_DOMAIN_TRAITS_ONLY_YML: &'static str =
        include_str!("../rules/clean-architecture/CA003_domain-traits-only.yml");
    const RULES_CLEAN_ARCHITECTURE_CA004_HANDLER_DEPENDENCY_INJECTION_YML: &'static str =
        include_str!("../rules/clean-architecture/CA004_handler-dependency-injection.yml");
    const RULES_CLEAN_ARCHITECTURE_CA005_ENTITY_IDENTITY_YML: &'static str =
        include_str!("../rules/clean-architecture/CA005_entity-identity.yml");
    const RULES_CLEAN_ARCHITECTURE_CA006_VALUE_OBJECT_IMMUTABILITY_YML: &'static str =
        include_str!("../rules/clean-architecture/CA006_value-object-immutability.yml");
    const RULES_CLEAN_ARCHITECTURE_CA007_INFRASTRUCTURE_CONCRETE_IMPORTS_YML: &'static str =
        include_str!("../rules/clean-architecture/CA007_infrastructure-concrete-imports.yml");
    const RULES_CLEAN_ARCHITECTURE_CA008_APPLICATION_PORT_IMPORTS_YML: &'static str =
        include_str!("../rules/clean-architecture/CA008_application-port-imports.yml");
    const RULES_CLEAN_ARCHITECTURE_CA009_INFRASTRUCTURE_NO_APPLICATION_YML: &'static str =
        include_str!("../rules/clean-architecture/CA009_infrastructure-no-application.yml");
    const RULES_CLEAN_ARCHITECTURE_CA010_PROVIDERS_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA010_providers-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA011_INFRASTRUCTURE_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA011_infrastructure-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA012_SERVER_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA012_server-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA013_MCB_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA013_mcb-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA014_VALIDATE_BOUNDARIES_YML: &'static str =
        include_str!("../rules/clean-architecture/CA014_validate-boundaries.yml");
    const RULES_CLEAN_ARCHITECTURE_CA017_DI_PATTERNS_YML: &'static str =
        include_str!("../rules/clean-architecture/CA017_di-patterns.yml");
    const RULES_CLEAN_ARCHITECTURE_LAYER001_LAYER_FLOW_RULES_YML: &'static str =
        include_str!("../rules/clean-architecture/LAYER001_layer-flow-rules.yml");
    const RULES_CLEAN_ARCHITECTURE_PORT001_PORT_ADAPTER_CONFIG_YML: &'static str =
        include_str!("../rules/clean-architecture/PORT001_port-adapter-config.yml");
    const RULES_DI_SHAKU001_COMPONENT_DERIVE_YML: &'static str =
        include_str!("../rules/di/SHAKU001_component-derive.yml");
    const RULES_DI_SHAKU002_INTERFACE_ANNOTATION_YML: &'static str =
        include_str!("../rules/di/SHAKU002_interface-annotation.yml");
    const RULES_DI_SHAKU003_CONCRETE_TYPE_HANDLER_YML: &'static str =
        include_str!("../rules/di/SHAKU003_concrete-type-handler.yml");
    const RULES_DOCUMENTATION_DOC001_COMPLETENESS_YML: &'static str =
        include_str!("../rules/documentation/DOC001_completeness.yml");
    const RULES_DUPLICATION_DUP001_EXACT_CLONE_YML: &'static str =
        include_str!("../rules/duplication/DUP001_exact-clone.yml");
    const RULES_DUPLICATION_DUP002_RENAMED_CLONE_YML: &'static str =
        include_str!("../rules/duplication/DUP002_renamed-clone.yml");
    const RULES_DUPLICATION_DUP003_GAPPED_CLONE_YML: &'static str =
        include_str!("../rules/duplication/DUP003_gapped-clone.yml");
    const RULES_IMPLEMENTATION_IMPL001_QUALITY_YML: &'static str =
        include_str!("../rules/implementation/IMPL001_quality.yml");
    const RULES_METRICS_METRIC001_COGNITIVE_COMPLEXITY_YML: &'static str =
        include_str!("../rules/metrics/METRIC001_cognitive-complexity.yml");
    const RULES_METRICS_METRIC002_FUNCTION_LENGTH_YML: &'static str =
        include_str!("../rules/metrics/METRIC002_function-length.yml");
    const RULES_METRICS_METRIC003_NESTING_DEPTH_YML: &'static str =
        include_str!("../rules/metrics/METRIC003_nesting-depth.yml");
    const RULES_METRICS_METRIC004_CYCLOMATIC_COMPLEXITY_YML: &'static str =
        include_str!("../rules/metrics/METRIC004_cyclomatic-complexity.yml");
    const RULES_METRICS_METRIC005_HALSTEAD_VOLUME_YML: &'static str =
        include_str!("../rules/metrics/METRIC005_halstead-volume.yml");
    const RULES_METRICS_METRIC006_MAINTAINABILITY_INDEX_YML: &'static str =
        include_str!("../rules/metrics/METRIC006_maintainability-index.yml");
    const RULES_MIGRATION_CTOR001_SHAKU_MIGRATION_YML: &'static str =
        include_str!("../rules/migration/CTOR001_shaku-migration.yml");
    const RULES_MIGRATION_CTOR002_CONSTRUCTOR_INJECTION_YML: &'static str =
        include_str!("../rules/migration/CTOR002_constructor-injection.yml");
    const RULES_MIGRATION_CTOR003_MANUAL_SERVICE_COMPOSITION_YML: &'static str =
        include_str!("../rules/migration/CTOR003_manual-service-composition.yml");
    const RULES_MIGRATION_FIGMENT001_FIGMENT_MIGRATION_YML: &'static str =
        include_str!("../rules/migration/FIGMENT001_figment-migration.yml");
    const RULES_MIGRATION_FIGMENT002_FIGMENT_PATTERN_YML: &'static str =
        include_str!("../rules/migration/FIGMENT002_figment-pattern.yml");
    const RULES_MIGRATION_FIGMENT003_FIGMENT_PROFILE_SUPPORT_YML: &'static str =
        include_str!("../rules/migration/FIGMENT003_figment-profile-support.yml");
    const RULES_MIGRATION_LINKME001_INVENTORY_MIGRATION_YML: &'static str =
        include_str!("../rules/migration/LINKME001_inventory-migration.yml");
    const RULES_MIGRATION_LINKME002_LINKME_SLICE_DECLARATION_YML: &'static str =
        include_str!("../rules/migration/LINKME002_linkme-slice-declaration.yml");
    const RULES_MIGRATION_LINKME003_LINKME_SLICE_USAGE_YML: &'static str =
        include_str!("../rules/migration/LINKME003_linkme-slice-usage.yml");
    const RULES_MIGRATION_ROCKET001_ROCKET_MIGRATION_YML: &'static str =
        include_str!("../rules/migration/ROCKET001_rocket-migration.yml");
    const RULES_MIGRATION_ROCKET002_ROCKET_ATTRIBUTE_HANDLERS_YML: &'static str =
        include_str!("../rules/migration/ROCKET002_rocket-attribute-handlers.yml");
    const RULES_MIGRATION_ROCKET003_ROCKET_ROUTE_ORGANIZATION_YML: &'static str =
        include_str!("../rules/migration/ROCKET003_rocket-route-organization.yml");
    const RULES_ORGANIZATION_ORG001_STRUCTURE_YML: &'static str =
        include_str!("../rules/organization/ORG001_structure.yml");
    const RULES_ORGANIZATION_ORG015_ADAPTER_LOCATION_YML: &'static str =
        include_str!("../rules/organization/ORG015_adapter-location.yml");
    const RULES_ORGANIZATION_ORG017_HANDLER_LOCATION_YML: &'static str =
        include_str!("../rules/organization/ORG017_handler-location.yml");
    const RULES_ORGANIZATION_ORG018_PORT_LOCATION_YML: &'static str =
        include_str!("../rules/organization/ORG018_port-location.yml");
    const RULES_ORGANIZATION_ORG019_TRAIT_PLACEMENT_YML: &'static str =
        include_str!("../rules/organization/ORG019_trait-placement.yml");
    const RULES_ORGANIZATION_ORG020_DOMAIN_ADAPTERS_YML: &'static str =
        include_str!("../rules/organization/ORG020_domain-adapters.yml");
    const RULES_ORGANIZATION_ORG021_INFRA_PORTS_YML: &'static str =
        include_str!("../rules/organization/ORG021_infra-ports.yml");
    const RULES_ORGANIZATION_ORG022_SCATTERED_CONFIG_YML: &'static str =
        include_str!("../rules/organization/ORG022_scattered-config.yml");
    const RULES_ORGANIZATION_ORG023_NESTED_ERRORS_YML: &'static str =
        include_str!("../rules/organization/ORG023_nested-errors.yml");
    const RULES_ORGANIZATION_VIS001_VISIBILITY_CONFIG_YML: &'static str =
        include_str!("../rules/organization/VIS001_visibility-config.yml");
    const RULES_PERFORMANCE_ASYNC001_PATTERNS_YML: &'static str =
        include_str!("../rules/performance/ASYNC001_patterns.yml");
    const RULES_QUALITY_QUAL001_NO_UNWRAP_YML: &'static str =
        include_str!("../rules/quality/QUAL001_no-unwrap.yml");
    const RULES_QUALITY_QUAL002_NO_EXPECT_YML: &'static str =
        include_str!("../rules/quality/QUAL002_no-expect.yml");
    const RULES_QUALITY_QUAL004_FUNCTION_SIZE_LIMIT_YML: &'static str =
        include_str!("../rules/quality/QUAL004_function-size-limit.yml");
    const RULES_QUALITY_QUAL005_RUFF_IMPORTS_YML: &'static str =
        include_str!("../rules/quality/QUAL005_ruff-imports.yml");
    const RULES_QUALITY_QUAL006_FILE_SIZE_LIMIT_YML: &'static str =
        include_str!("../rules/quality/QUAL006_file-size-limit.yml");
    const RULES_REFACTORING_REF001_REFACTORING_CONFIG_YML: &'static str =
        include_str!("../rules/refactoring/REF001_refactoring-config.yml");
    const RULES_SOLID_SOLID001_TRAIT_METHODS_YML: &'static str =
        include_str!("../rules/solid/SOLID001_trait-methods.yml");
    const RULES_SOLID_SOLID002_IMPL_METHODS_YML: &'static str =
        include_str!("../rules/solid/SOLID002_impl-methods.yml");
    const RULES_SOLID_SOLID003_MATCH_COMPLEXITY_YML: &'static str =
        include_str!("../rules/solid/SOLID003_match-complexity.yml");
    const RULES_TEMPLATES_CARGO_DEPENDENCY_TEMPLATE_YML: &'static str =
        include_str!("../rules/templates/cargo-dependency-template.yml");
    const RULES_TEMPLATES_CODE_PATTERN_TEMPLATE_YML: &'static str =
        include_str!("../rules/templates/code-pattern-template.yml");
    const RULES_TEMPLATES_IMPORT_CHECK_TEMPLATE_YML: &'static str =
        include_str!("../rules/templates/import-check-template.yml");
    const RULES_TESTING_TEST001_ORGANIZATION_YML: &'static str =
        include_str!("../rules/testing/TEST001_organization.yml");

    /// All embedded YAML files including templates.
    #[must_use]
    pub fn all_yaml() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "rules/clean-architecture/CA001_domain-independence.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA001_DOMAIN_INDEPENDENCE_YML,
            ),
            (
                "rules/clean-architecture/CA002_application-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA002_APPLICATION_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA003_domain-traits-only.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA003_DOMAIN_TRAITS_ONLY_YML,
            ),
            (
                "rules/clean-architecture/CA004_handler-dependency-injection.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA004_HANDLER_DEPENDENCY_INJECTION_YML,
            ),
            (
                "rules/clean-architecture/CA005_entity-identity.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA005_ENTITY_IDENTITY_YML,
            ),
            (
                "rules/clean-architecture/CA006_value-object-immutability.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA006_VALUE_OBJECT_IMMUTABILITY_YML,
            ),
            (
                "rules/clean-architecture/CA007_infrastructure-concrete-imports.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA007_INFRASTRUCTURE_CONCRETE_IMPORTS_YML,
            ),
            (
                "rules/clean-architecture/CA008_application-port-imports.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA008_APPLICATION_PORT_IMPORTS_YML,
            ),
            (
                "rules/clean-architecture/CA009_infrastructure-no-application.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA009_INFRASTRUCTURE_NO_APPLICATION_YML,
            ),
            (
                "rules/clean-architecture/CA010_providers-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA010_PROVIDERS_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA011_infrastructure-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA011_INFRASTRUCTURE_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA012_server-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA012_SERVER_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA013_mcb-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA013_MCB_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA014_validate-boundaries.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA014_VALIDATE_BOUNDARIES_YML,
            ),
            (
                "rules/clean-architecture/CA017_di-patterns.yml",
                Self::RULES_CLEAN_ARCHITECTURE_CA017_DI_PATTERNS_YML,
            ),
            (
                "rules/clean-architecture/LAYER001_layer-flow-rules.yml",
                Self::RULES_CLEAN_ARCHITECTURE_LAYER001_LAYER_FLOW_RULES_YML,
            ),
            (
                "rules/clean-architecture/PORT001_port-adapter-config.yml",
                Self::RULES_CLEAN_ARCHITECTURE_PORT001_PORT_ADAPTER_CONFIG_YML,
            ),
            (
                "rules/di/SHAKU001_component-derive.yml",
                Self::RULES_DI_SHAKU001_COMPONENT_DERIVE_YML,
            ),
            (
                "rules/di/SHAKU002_interface-annotation.yml",
                Self::RULES_DI_SHAKU002_INTERFACE_ANNOTATION_YML,
            ),
            (
                "rules/di/SHAKU003_concrete-type-handler.yml",
                Self::RULES_DI_SHAKU003_CONCRETE_TYPE_HANDLER_YML,
            ),
            (
                "rules/documentation/DOC001_completeness.yml",
                Self::RULES_DOCUMENTATION_DOC001_COMPLETENESS_YML,
            ),
            (
                "rules/duplication/DUP001_exact-clone.yml",
                Self::RULES_DUPLICATION_DUP001_EXACT_CLONE_YML,
            ),
            (
                "rules/duplication/DUP002_renamed-clone.yml",
                Self::RULES_DUPLICATION_DUP002_RENAMED_CLONE_YML,
            ),
            (
                "rules/duplication/DUP003_gapped-clone.yml",
                Self::RULES_DUPLICATION_DUP003_GAPPED_CLONE_YML,
            ),
            (
                "rules/implementation/IMPL001_quality.yml",
                Self::RULES_IMPLEMENTATION_IMPL001_QUALITY_YML,
            ),
            (
                "rules/metrics/METRIC001_cognitive-complexity.yml",
                Self::RULES_METRICS_METRIC001_COGNITIVE_COMPLEXITY_YML,
            ),
            (
                "rules/metrics/METRIC002_function-length.yml",
                Self::RULES_METRICS_METRIC002_FUNCTION_LENGTH_YML,
            ),
            (
                "rules/metrics/METRIC003_nesting-depth.yml",
                Self::RULES_METRICS_METRIC003_NESTING_DEPTH_YML,
            ),
            (
                "rules/metrics/METRIC004_cyclomatic-complexity.yml",
                Self::RULES_METRICS_METRIC004_CYCLOMATIC_COMPLEXITY_YML,
            ),
            (
                "rules/metrics/METRIC005_halstead-volume.yml",
                Self::RULES_METRICS_METRIC005_HALSTEAD_VOLUME_YML,
            ),
            (
                "rules/metrics/METRIC006_maintainability-index.yml",
                Self::RULES_METRICS_METRIC006_MAINTAINABILITY_INDEX_YML,
            ),
            (
                "rules/migration/CTOR001_shaku-migration.yml",
                Self::RULES_MIGRATION_CTOR001_SHAKU_MIGRATION_YML,
            ),
            (
                "rules/migration/CTOR002_constructor-injection.yml",
                Self::RULES_MIGRATION_CTOR002_CONSTRUCTOR_INJECTION_YML,
            ),
            (
                "rules/migration/CTOR003_manual-service-composition.yml",
                Self::RULES_MIGRATION_CTOR003_MANUAL_SERVICE_COMPOSITION_YML,
            ),
            (
                "rules/migration/FIGMENT001_figment-migration.yml",
                Self::RULES_MIGRATION_FIGMENT001_FIGMENT_MIGRATION_YML,
            ),
            (
                "rules/migration/FIGMENT002_figment-pattern.yml",
                Self::RULES_MIGRATION_FIGMENT002_FIGMENT_PATTERN_YML,
            ),
            (
                "rules/migration/FIGMENT003_figment-profile-support.yml",
                Self::RULES_MIGRATION_FIGMENT003_FIGMENT_PROFILE_SUPPORT_YML,
            ),
            (
                "rules/migration/LINKME001_inventory-migration.yml",
                Self::RULES_MIGRATION_LINKME001_INVENTORY_MIGRATION_YML,
            ),
            (
                "rules/migration/LINKME002_linkme-slice-declaration.yml",
                Self::RULES_MIGRATION_LINKME002_LINKME_SLICE_DECLARATION_YML,
            ),
            (
                "rules/migration/LINKME003_linkme-slice-usage.yml",
                Self::RULES_MIGRATION_LINKME003_LINKME_SLICE_USAGE_YML,
            ),
            (
                "rules/migration/ROCKET001_rocket-migration.yml",
                Self::RULES_MIGRATION_ROCKET001_ROCKET_MIGRATION_YML,
            ),
            (
                "rules/migration/ROCKET002_rocket-attribute-handlers.yml",
                Self::RULES_MIGRATION_ROCKET002_ROCKET_ATTRIBUTE_HANDLERS_YML,
            ),
            (
                "rules/migration/ROCKET003_rocket-route-organization.yml",
                Self::RULES_MIGRATION_ROCKET003_ROCKET_ROUTE_ORGANIZATION_YML,
            ),
            (
                "rules/organization/ORG001_structure.yml",
                Self::RULES_ORGANIZATION_ORG001_STRUCTURE_YML,
            ),
            (
                "rules/organization/ORG015_adapter-location.yml",
                Self::RULES_ORGANIZATION_ORG015_ADAPTER_LOCATION_YML,
            ),
            (
                "rules/organization/ORG017_handler-location.yml",
                Self::RULES_ORGANIZATION_ORG017_HANDLER_LOCATION_YML,
            ),
            (
                "rules/organization/ORG018_port-location.yml",
                Self::RULES_ORGANIZATION_ORG018_PORT_LOCATION_YML,
            ),
            (
                "rules/organization/ORG019_trait-placement.yml",
                Self::RULES_ORGANIZATION_ORG019_TRAIT_PLACEMENT_YML,
            ),
            (
                "rules/organization/ORG020_domain-adapters.yml",
                Self::RULES_ORGANIZATION_ORG020_DOMAIN_ADAPTERS_YML,
            ),
            (
                "rules/organization/ORG021_infra-ports.yml",
                Self::RULES_ORGANIZATION_ORG021_INFRA_PORTS_YML,
            ),
            (
                "rules/organization/ORG022_scattered-config.yml",
                Self::RULES_ORGANIZATION_ORG022_SCATTERED_CONFIG_YML,
            ),
            (
                "rules/organization/ORG023_nested-errors.yml",
                Self::RULES_ORGANIZATION_ORG023_NESTED_ERRORS_YML,
            ),
            (
                "rules/organization/VIS001_visibility-config.yml",
                Self::RULES_ORGANIZATION_VIS001_VISIBILITY_CONFIG_YML,
            ),
            (
                "rules/performance/ASYNC001_patterns.yml",
                Self::RULES_PERFORMANCE_ASYNC001_PATTERNS_YML,
            ),
            (
                "rules/quality/QUAL001_no-unwrap.yml",
                Self::RULES_QUALITY_QUAL001_NO_UNWRAP_YML,
            ),
            (
                "rules/quality/QUAL002_no-expect.yml",
                Self::RULES_QUALITY_QUAL002_NO_EXPECT_YML,
            ),
            (
                "rules/quality/QUAL004_function-size-limit.yml",
                Self::RULES_QUALITY_QUAL004_FUNCTION_SIZE_LIMIT_YML,
            ),
            (
                "rules/quality/QUAL005_ruff-imports.yml",
                Self::RULES_QUALITY_QUAL005_RUFF_IMPORTS_YML,
            ),
            (
                "rules/quality/QUAL006_file-size-limit.yml",
                Self::RULES_QUALITY_QUAL006_FILE_SIZE_LIMIT_YML,
            ),
            (
                "rules/refactoring/REF001_refactoring-config.yml",
                Self::RULES_REFACTORING_REF001_REFACTORING_CONFIG_YML,
            ),
            (
                "rules/solid/SOLID001_trait-methods.yml",
                Self::RULES_SOLID_SOLID001_TRAIT_METHODS_YML,
            ),
            (
                "rules/solid/SOLID002_impl-methods.yml",
                Self::RULES_SOLID_SOLID002_IMPL_METHODS_YML,
            ),
            (
                "rules/solid/SOLID003_match-complexity.yml",
                Self::RULES_SOLID_SOLID003_MATCH_COMPLEXITY_YML,
            ),
            (
                "rules/templates/cargo-dependency-template.yml",
                Self::RULES_TEMPLATES_CARGO_DEPENDENCY_TEMPLATE_YML,
            ),
            (
                "rules/templates/code-pattern-template.yml",
                Self::RULES_TEMPLATES_CODE_PATTERN_TEMPLATE_YML,
            ),
            (
                "rules/templates/import-check-template.yml",
                Self::RULES_TEMPLATES_IMPORT_CHECK_TEMPLATE_YML,
            ),
            (
                "rules/testing/TEST001_organization.yml",
                Self::RULES_TESTING_TEST001_ORGANIZATION_YML,
            ),
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
