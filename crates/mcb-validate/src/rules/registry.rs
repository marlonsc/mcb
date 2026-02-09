//! Declarative Rule Registry
//!
//! Rules are defined as data structures for easy configuration and extension.

use std::collections::HashMap;

use crate::violation_trait::{Severity, ViolationCategory};

/// Declarative rule definition
#[derive(Debug, Clone)]
pub struct Rule {
    /// Unique rule identifier (e.g., "CA001")
    pub id: String,
    /// Human-readable rule name
    pub name: String,
    /// Category for grouping in reports
    pub category: ViolationCategory,
    /// Default severity level
    pub default_severity: Severity,
    /// Description of what the rule checks
    pub description: String,
    /// Why this rule matters
    pub rationale: String,
    /// Whether rule is enabled by default
    pub enabled: bool,
    /// Rule-specific configuration values
    pub config: HashMap<String, RuleConfigValue>,
}

/// Configuration value types for rules
#[derive(Debug, Clone)]
pub enum RuleConfigValue {
    /// Single string value
    String(String),
    /// List of string values
    StringList(Vec<String>),
    /// Integer numeric value
    Number(i64),
    /// Boolean flag value
    Boolean(bool),
}

impl From<&str> for RuleConfigValue {
    fn from(s: &str) -> Self {
        RuleConfigValue::String(s.to_string())
    }
}

impl From<Vec<&str>> for RuleConfigValue {
    fn from(v: Vec<&str>) -> Self {
        RuleConfigValue::StringList(v.into_iter().map(String::from).collect())
    }
}

impl From<i64> for RuleConfigValue {
    fn from(n: i64) -> Self {
        RuleConfigValue::Number(n)
    }
}

impl From<bool> for RuleConfigValue {
    fn from(b: bool) -> Self {
        RuleConfigValue::Boolean(b)
    }
}

/// Registry holding all defined rules
#[derive(Debug, Default)]
pub struct RuleRegistry {
    /// List of registered rules
    rules: Vec<Rule>,
}

impl RuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the registry
    pub fn register(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Get all rules
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Get rules by category
    pub fn rules_by_category(&self, category: ViolationCategory) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Get a rule by ID
    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Check if a rule is enabled
    pub fn is_enabled(&self, id: &str) -> bool {
        self.get(id).is_some_and(|r| r.enabled)
    }

    /// Create registry with all standard rules
    pub fn standard() -> Self {
        let mut registry = Self::new();

        for rule in clean_architecture_rules() {
            registry.register(rule);
        }

        for rule in layer_boundary_rules() {
            registry.register(rule);
        }

        for rule in quality_rules() {
            registry.register(rule);
        }

        for rule in solid_rules() {
            registry.register(rule);
        }

        for rule in linkme_rules() {
            registry.register(rule);
        }

        for rule in constructor_injection_rules() {
            registry.register(rule);
        }

        for rule in figment_rules() {
            registry.register(rule);
        }

        for rule in rocket_rules() {
            registry.register(rule);
        }

        registry
    }
}

/// Clean Architecture rules
pub fn clean_architecture_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "CA001".into(),
            name: "Domain Layer Independence".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Domain crate must have zero internal dependencies".into(),
            rationale: "Domain layer contains pure business logic independent of frameworks".into(),
            enabled: true,
            config: HashMap::from([
                ("crate".into(), RuleConfigValue::from("")),
                (
                    "forbidden_prefixes".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
            ]),
        },
        Rule {
            id: "CA002".into(),
            name: "Application Layer Boundaries".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Application crate only depends on domain".into(),
            rationale: "Application orchestrates domain logic without infrastructure details"
                .into(),
            enabled: true,
            config: HashMap::from([
                ("crate".into(), RuleConfigValue::from("")),
                ("allowed".into(), RuleConfigValue::from(vec![] as Vec<&str>)),
            ]),
        },
        Rule {
            id: "CA003".into(),
            name: "Domain Contains Only Traits".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Domain layer should only contain traits and value objects".into(),
            rationale: "Implementations belong in infrastructure/providers layers".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "CA004".into(),
            name: "Handler Dependency Injection".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Handlers must receive dependencies via injection, not create them".into(),
            rationale: "Direct instantiation couples handlers to concrete implementations".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "CA005".into(),
            name: "Entity Identity Marker".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Warning,
            description: "Entities should have an identity field (id, uuid)".into(),
            rationale: "Entities are defined by their identity, not their attributes".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "CA006".into(),
            name: "Value Object Immutability".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Value objects must be immutable".into(),
            rationale: "Value objects are defined by their attributes and should not change".into(),
            enabled: true,
            config: HashMap::new(),
        },
    ]
}

/// Layer boundary rules
pub fn layer_boundary_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "LAYER001".into(),
            name: "Forbidden Cargo Dependency".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Crate has forbidden dependency in Cargo.toml".into(),
            rationale: "Layer boundaries must be enforced at the dependency level".into(),
            enabled: true,
            config: HashMap::from([
                (
                    "domain_deps".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
                (
                    "application_deps".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
                (
                    "providers_deps".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
                (
                    "infrastructure_deps".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
                (
                    "server_deps".into(),
                    RuleConfigValue::from(vec![] as Vec<&str>),
                ),
            ]),
        },
        Rule {
            id: "LAYER002".into(),
            name: "Circular Dependency".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Error,
            description: "Circular dependency detected between crates".into(),
            rationale: "Circular dependencies indicate architectural problems".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "LAYER003".into(),
            name: "Domain External Dependency".into(),
            category: ViolationCategory::Architecture,
            default_severity: Severity::Warning,
            description: "Domain layer imports external framework crate".into(),
            rationale: "Domain should only use std, serde, thiserror".into(),
            enabled: true,
            config: HashMap::from([(
                "allowed_external".into(),
                RuleConfigValue::from(vec!["std", "serde", "thiserror", "uuid", "chrono"]),
            )]),
        },
    ]
}

/// Quality rules
pub fn quality_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "QUAL001".into(),
            name: "No Unwrap in Production".into(),
            category: ViolationCategory::Quality,
            default_severity: Severity::Error,
            description: "Production code should not use .unwrap()".into(),
            rationale: "Unwrap can cause panics; use ? operator or proper error handling".into(),
            enabled: true,
            config: HashMap::from([("allow_in_tests".into(), RuleConfigValue::from(true))]),
        },
        Rule {
            id: "QUAL002".into(),
            name: "No Expect in Production".into(),
            category: ViolationCategory::Quality,
            default_severity: Severity::Warning,
            description: "Production code should not use .expect()".into(),
            rationale: "Expect can cause panics; use ? operator with context".into(),
            enabled: true,
            config: HashMap::from([("allow_in_tests".into(), RuleConfigValue::from(true))]),
        },
        Rule {
            id: "QUAL003".into(),
            name: "File Size Limit".into(),
            category: ViolationCategory::Quality,
            default_severity: Severity::Warning,
            description: "File exceeds maximum line count".into(),
            rationale: "Large files are harder to maintain and understand".into(),
            enabled: true,
            config: HashMap::from([("max_lines".into(), RuleConfigValue::from(500i64))]),
        },
        Rule {
            id: "QUAL004".into(),
            name: "Function Size Limit".into(),
            category: ViolationCategory::Quality,
            default_severity: Severity::Warning,
            description: "Function exceeds maximum line count".into(),
            rationale: "Large functions violate Single Responsibility Principle".into(),
            enabled: true,
            config: HashMap::from([("max_lines".into(), RuleConfigValue::from(50i64))]),
        },
    ]
}

/// SOLID principle rules
pub fn solid_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "SOLID001".into(),
            name: "Single Responsibility - Trait Methods".into(),
            category: ViolationCategory::Solid,
            default_severity: Severity::Warning,
            description: "Trait has too many methods".into(),
            rationale: "Traits with many methods violate Interface Segregation Principle".into(),
            enabled: true,
            config: HashMap::from([("max_methods".into(), RuleConfigValue::from(10i64))]),
        },
        Rule {
            id: "SOLID002".into(),
            name: "Single Responsibility - Impl Methods".into(),
            category: ViolationCategory::Solid,
            default_severity: Severity::Warning,
            description: "Impl block has too many methods".into(),
            rationale: "Large impl blocks suggest the struct has too many responsibilities".into(),
            enabled: true,
            config: HashMap::from([("max_methods".into(), RuleConfigValue::from(15i64))]),
        },
        Rule {
            id: "SOLID003".into(),
            name: "Match Arm Complexity".into(),
            category: ViolationCategory::Solid,
            default_severity: Severity::Info,
            description: "Match expression has too many arms".into(),
            rationale: "Consider using polymorphism or strategy pattern".into(),
            enabled: true,
            config: HashMap::from([("max_arms".into(), RuleConfigValue::from(10i64))]),
        },
    ]
}

/// Linkme distributed slice rules (v0.2.0)
pub fn linkme_rules() -> Vec<Rule> {
    vec![
        migration_rule(
            "LINKME001",
            "Inventory Migration Required",
            ViolationCategory::DependencyInjection,
            "Code still uses inventory::submit! or inventory::collect!",
            "inventory crate is being replaced by linkme for simpler plugin registration",
        ),
        Rule {
            id: "LINKME002".into(),
            name: "Linkme Slice Declaration".into(),
            category: ViolationCategory::DependencyInjection,
            default_severity: Severity::Warning,
            description: "Provider registry missing #[linkme::distributed_slice] declaration"
                .into(),
            rationale: "All provider registries must use linkme distributed slices".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "LINKME003".into(),
            name: "Linkme Slice Usage".into(),
            category: ViolationCategory::DependencyInjection,
            default_severity: Severity::Warning,
            description:
                "Provider registration missing #[linkme::distributed_slice(NAME)] attribute".into(),
            rationale: "All provider implementations must be registered via linkme slices".into(),
            enabled: true,
            config: HashMap::new(),
        },
    ]
}

/// Constructor injection rules (v0.2.0)
pub fn constructor_injection_rules() -> Vec<Rule> {
    vec![
        migration_rule(
            "CTOR001",
            "Shaku Migration Required",
            ViolationCategory::DependencyInjection,
            "Code still uses Shaku DI patterns (#[derive(Component)], module! macro)",
            "Shaku DI is being replaced by direct constructor injection for simplicity",
        ),
        Rule {
            id: "CTOR002".into(),
            name: "Constructor Injection Pattern".into(),
            category: ViolationCategory::DependencyInjection,
            default_severity: Severity::Warning,
            description:
                "Service implementation missing constructor that accepts Arc<dyn Trait> parameters"
                    .into(),
            rationale: "All services must use constructor injection for dependency management"
                .into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "CTOR003".into(),
            name: "Manual Service Composition".into(),
            category: ViolationCategory::DependencyInjection,
            default_severity: Severity::Info,
            description: "Service instantiation should happen in bootstrap/container functions"
                .into(),
            rationale: "Dependency wiring should be centralized and explicit".into(),
            enabled: true,
            config: HashMap::new(),
        },
    ]
}

/// Figment configuration rules (v0.2.0)
pub fn figment_rules() -> Vec<Rule> {
    vec![
        migration_rule(
            "FIGMENT001",
            "Config Crate Migration Required",
            ViolationCategory::Configuration,
            "Code still uses config crate (Config::builder(), Environment, File)",
            "config crate is being replaced by Figment for unified configuration",
        ),
        Rule {
            id: "FIGMENT002".into(),
            name: "Figment Pattern Usage".into(),
            category: ViolationCategory::Configuration,
            default_severity: Severity::Warning,
            description:
                "Configuration loading should use Figment::new().merge().extract() pattern".into(),
            rationale: "Figment provides unified configuration source handling".into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "FIGMENT003".into(),
            name: "Profile Support".into(),
            category: ViolationCategory::Configuration,
            default_severity: Severity::Info,
            description: "Consider adding profile-based configuration (dev/prod)".into(),
            rationale: "Figment enables easy environment-specific configuration".into(),
            enabled: true,
            config: HashMap::new(),
        },
    ]
}

/// Rocket routing rules (v0.2.0)
pub fn rocket_rules() -> Vec<Rule> {
    vec![
        migration_rule(
            "ROCKET001",
            "Axum Migration Required",
            ViolationCategory::WebFramework,
            "Code still uses Axum routing patterns (Router::new(), axum::routing::*)",
            "Axum is being replaced by Rocket for attribute-based routing simplicity",
        ),
        Rule {
            id: "ROCKET002".into(),
            name: "Attribute-Based Handlers".into(),
            category: ViolationCategory::WebFramework,
            default_severity: Severity::Warning,
            description: "HTTP handlers should use Rocket attribute macros (#[get], #[post], etc.)"
                .into(),
            rationale: "Attribute-based routing reduces boilerplate and improves readability"
                .into(),
            enabled: true,
            config: HashMap::new(),
        },
        Rule {
            id: "ROCKET003".into(),
            name: "Rocket Route Organization".into(),
            category: ViolationCategory::WebFramework,
            default_severity: Severity::Info,
            description: "Routes should be organized in feature modules with routes![] macro"
                .into(),
            rationale: "Clean route organization improves maintainability".into(),
            enabled: true,
            config: HashMap::new(),
        },
    ]
}

/// Helper: Create migration rule with standard config
fn migration_rule(
    id: &str,
    name: &str,
    category: ViolationCategory,
    description: &str,
    rationale: &str,
) -> Rule {
    Rule {
        id: id.into(),
        name: name.into(),
        category,
        default_severity: Severity::Error,
        description: description.into(),
        rationale: rationale.into(),
        enabled: true,
        config: HashMap::from([("migration_deadline".into(), RuleConfigValue::from("v0.2.0"))]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_registry_creation() {
        let registry = RuleRegistry::standard();
        assert!(!registry.rules().is_empty());
    }

    #[test]
    fn test_rule_lookup() {
        let registry = RuleRegistry::standard();
        let rule = registry.get("CA001");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "Domain Layer Independence");
    }

    #[test]
    fn test_rules_by_category() {
        let registry = RuleRegistry::standard();
        let arch_rules = registry.rules_by_category(ViolationCategory::Architecture);
        assert!(!arch_rules.is_empty());
    }

    #[test]
    fn test_rule_enabled_check() {
        let registry = RuleRegistry::standard();
        assert!(registry.is_enabled("CA001"));
    }
}
