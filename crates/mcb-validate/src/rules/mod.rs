//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Rule Registry System
//!
//! Provides declarative rule definitions and registry management.

pub mod rule_types;
pub mod templates;
pub mod yaml_loader;
pub mod yaml_validator;

pub use rule_types::{AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule};
pub use templates::TemplateEngine;
pub use yaml_loader::YamlRuleLoader;
pub use yaml_validator::YamlRuleValidator;
