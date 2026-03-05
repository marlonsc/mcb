//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Linter Integration Module
//!
//! Integrates external linters (Ruff, Clippy) as first-layer validation
//! that feeds into the unified violation reporting system.

pub mod engine;
pub mod executor;
pub mod parsers;
pub mod runners;
pub mod types;

pub use engine::LinterEngine;
pub use executor::YamlRuleExecutor;
pub use runners::{ClippyLinter, RuffLinter};
pub use types::{LintViolation, LinterType};
