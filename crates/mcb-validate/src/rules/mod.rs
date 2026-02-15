//! Rule Registry System
//!
//! Provides declarative rule definitions and registry management.

pub mod templates;
/// Shared utility functions for rules.
pub mod utils;
pub mod yaml_loader;
pub mod yaml_validator;

pub use templates::TemplateEngine;
pub use yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use yaml_validator::YamlRuleValidator;
