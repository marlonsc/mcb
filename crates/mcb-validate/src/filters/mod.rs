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

/// Parser for `Cargo.toml` dependencies.
pub mod dependency_parser;
/// Matching system for file and directory glob patterns.
pub mod file_matcher;
/// Detection system for programming languages in source files.
pub mod language_detector;
/// High-level rule filter execution and coordination.
pub mod rule_filters;

pub use dependency_parser::{
    CargoDependencyParser, CrateDependencies, DependencyInfo, WorkspaceDependencies,
};
pub use file_matcher::FilePatternMatcher;
pub use language_detector::{LanguageDetector, LanguageId, language_from_rca, language_to_rca};
pub use rule_filters::{ApplicabilityFilter, RuleFilterExecutor, RuleFilters};
