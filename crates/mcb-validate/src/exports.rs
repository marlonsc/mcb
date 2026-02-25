//! Centralized public API re-exports.
//!
//! All symbols that are part of the crate's public API are re-exported here.
//! lib.rs then does `pub use exports::*` so there is a single place for the re-export list.

pub use ast::*;
pub use config::*;
pub use embedded_rules::EmbeddedRules;
pub use engines::{HybridRuleEngine, RuleEngineType};
pub use generic_reporter::{GenericReport, GenericReporter, GenericSummary};
pub use linters::{
    ClippyLinter, LintViolation, LinterEngine, LinterType, RuffLinter, YamlRuleExecutor,
};
pub use mcb_domain::ports::ViolationEntry;
pub use metrics::*;
pub use rules::*;
pub use run_context::{FileInventorySource, InventoryEntry, ValidationRunContext};
pub use thresholds::{ValidationThresholds, thresholds};
pub use traits::{Validator, ValidatorRegistry, Violation, ViolationCategory};
pub use unified_registry::{RuleInfo, RuleOrigin, UnifiedRuleRegistry};
pub use validators::*;
