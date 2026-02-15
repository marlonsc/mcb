use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Categories for error patterns to classify the type of error encountered.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ErrorPatternCategory {
    /// Compilation errors.
    Compilation,
    /// Runtime errors.
    Runtime,
    /// Test execution errors.
    Test,
    /// Linting errors.
    Lint,
    /// Build process errors.
    Build,
    /// Configuration errors.
    Config,
    /// Network-related errors.
    Network,
    /// Other error types.
    Other,
}

impl ErrorPatternCategory {
    /// Returns the string representation of the error pattern category.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// Represents a recurring error pattern detected in a project.
///
/// An error pattern captures the signature, category, and metadata of errors
/// that occur repeatedly, enabling pattern recognition and solution tracking.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Unique identifier for this error pattern.
    pub id: String,
    /// Project ID this pattern belongs to.
    pub project_id: String,
    /// Signature/fingerprint of the error pattern for matching.
    pub pattern_signature: String,
    /// Human-readable description of the error pattern.
    pub description: String,
    /// Category of the error.
    pub category: ErrorPatternCategory,
    /// List of known solutions for this error pattern.
    pub solutions: Vec<String>,
    /// Files affected by this error pattern.
    pub affected_files: Vec<String>,
    /// Tags for categorizing and searching the pattern.
    pub tags: Vec<String>,
    /// Number of times this pattern has occurred.
    pub occurrence_count: i64,
    /// Timestamp when this pattern was first observed.
    pub first_seen_at: i64,
    /// Timestamp when this pattern was last observed.
    pub last_seen_at: i64,
    /// Optional embedding ID for semantic search.
    pub embedding_id: Option<String>,
}

/// Represents a match between an observed error and a known error pattern.
///
/// Tracks when an error pattern is detected, the confidence level, and whether
/// a solution was applied and successful.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPatternMatch {
    /// Unique identifier for this match record.
    pub id: String,
    /// ID of the error pattern that was matched.
    pub pattern_id: String,
    /// ID of the observation that triggered this match.
    pub observation_id: String,
    /// Confidence level of the match (0-100).
    pub confidence: i64,
    /// Index of the solution applied, if any.
    pub solution_applied: Option<i32>,
    /// Whether the applied solution was successful.
    pub resolution_successful: Option<bool>,
    /// Timestamp when the pattern was matched.
    pub matched_at: i64,
    /// Timestamp when the error was resolved, if applicable.
    pub resolved_at: Option<i64>,
}
