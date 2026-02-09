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

/// Helper: Create architecture rule with empty config
fn arch_rule(id: &str, name: &str, description: &str, rationale: &str, severity: Severity) -> Rule {
    Rule {
        id: id.into(),
        name: name.into(),
        category: ViolationCategory::Architecture,
        default_severity: severity,
        description: description.into(),
        rationale: rationale.into(),
        enabled: true,
        config: HashMap::new(),
    }
}

/// Helper: Create DI rule with empty config
fn di_rule(id: &str, name: &str, description: &str, rationale: &str, severity: Severity) -> Rule {
    Rule {
        id: id.into(),
        name: name.into(),
        category: ViolationCategory::DependencyInjection,
        default_severity: severity,
        description: description.into(),
        rationale: rationale.into(),
        enabled: true,
        config: HashMap::new(),
    }
}

/// Helper: Create config rule with empty config
fn config_rule(
    id: &str,
    name: &str,
    description: &str,
    rationale: &str,
    severity: Severity,
) -> Rule {
    Rule {
        id: id.into(),
        name: name.into(),
        category: ViolationCategory::Configuration,
        default_severity: severity,
        description: description.into(),
        rationale: rationale.into(),
        enabled: true,
        config: HashMap::new(),
    }
}

/// Helper: Create web framework rule with empty config
fn web_rule(id: &str, name: &str, description: &str, rationale: &str, severity: Severity) -> Rule {
    Rule {
        id: id.into(),
        name: name.into(),
        category: ViolationCategory::WebFramework,
        default_severity: severity,
        description: description.into(),
        rationale: rationale.into(),
        enabled: true,
        config: HashMap::new(),
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
        arch_rule(
            "CA003",
            "Domain Contains Only Traits",
            "Domain must contain only trait definitions, not implementations",
            "Ports (traits) belong in domain; adapters (impls) belong in infrastructure",
            Severity::Error,
        ),
        arch_rule(
            "CA004",
            "Infrastructure Depends on Application",
            "Infrastructure crate must depend only on application and domain",
            "Infrastructure implements application ports",
            Severity::Error,
        ),
        arch_rule(
            "CA005",
            "No Concrete Types in Domain",
            "Domain services should depend on traits, not concrete types",
            "Dependency Inversion Principle: depend on abstractions",
            Severity::Warning,
        ),
        arch_rule(
            "CA006",
            "Value Object Immutability",
            "Value objects must be immutable",
            "Value objects are defined by their attributes and should not change",
            Severity::Error,
        ),
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
        arch_rule(
            "LAYER002",
            "Circular Dependency",
            "Circular dependency detected between crates",
            "Circular dependencies indicate architectural problems",
            Severity::Error,
        ),
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
        di_rule(
            "LINKME002",
            "Linkme Slice Declaration",
            "Provider registry missing #[linkme::distributed_slice] declaration",
            "All provider registries must use linkme distributed slices",
            Severity::Warning,
        ),
        di_rule(
            "LINKME003",
            "Linkme Slice Usage",
            "Provider registration missing #[linkme::distributed_slice(NAME)] attribute",
            "All provider implementations must be registered via linkme slices",
            Severity::Warning,
        ),
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
        di_rule(
            "CTOR002",
            "Constructor Injection Pattern",
            "Service implementation missing constructor that accepts Arc<dyn Trait> parameters",
            "All services must use constructor injection for dependency management",
            Severity::Warning,
        ),
        di_rule(
            "CTOR003",
            "Manual Service Composition",
            "Service instantiation should happen in bootstrap/container functions",
            "Dependency wiring should be centralized and explicit",
            Severity::Info,
        ),
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
        config_rule(
            "FIGMENT002",
            "Figment Pattern Usage",
            "Configuration loading should use Figment::new().merge().extract() pattern",
            "Figment provides unified configuration source handling",
            Severity::Warning,
        ),
        config_rule(
            "FIGMENT003",
            "Profile Support",
            "Consider adding profile-based configuration (dev/prod)",
            "Figment enables easy environment-specific configuration",
            Severity::Info,
        ),
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
        web_rule(
            "ROCKET002",
            "Attribute-Based Handlers",
            "HTTP handlers should use Rocket attribute macros (#[get], #[post], etc.)",
            "Attribute-based routing reduces boilerplate and improves readability",
            Severity::Warning,
        ),
        web_rule(
            "ROCKET003",
            "Rocket Route Organization",
            "Routes should be organized in feature modules with routes![] macro",
            "Clean route organization improves maintainability",
            Severity::Info,
        ),
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
