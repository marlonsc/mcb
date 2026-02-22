//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Rule Filtering System
//!
//! Provides intelligent filtering of validation rules based on:
//! - File language detection
//! - Dependency analysis
//! - File patterns and exclusions
//!
//! This prevents rules from running on irrelevant files, improving performance
//! and reducing false positives.

pub mod dependency_parser;
pub mod file_matcher;
pub mod language_detector;
pub mod rule_filters;

pub use dependency_parser::{
    CargoDependencyParser, CrateDependencies, DependencyInfo, WorkspaceDependencies,
};
pub use file_matcher::FilePatternMatcher;
pub use language_detector::{LanguageDetector, LanguageId};
pub use rule_filters::{ApplicabilityFilter, RuleFilterExecutor, RuleFilters};
