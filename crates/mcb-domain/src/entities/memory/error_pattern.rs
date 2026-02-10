use serde::{Deserialize, Serialize};

/// Categories for error patterns to classify the type of error encountered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compilation => "compilation",
            Self::Runtime => "runtime",
            Self::Test => "test",
            Self::Lint => "lint",
            Self::Build => "build",
            Self::Config => "config",
            Self::Network => "network",
            Self::Other => "other",
        }
    }
}

impl std::str::FromStr for ErrorPatternCategory {
    /// Error type for parsing failures.
    type Err = String;

    /// Parses a string into an `ErrorPatternCategory`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compilation" => Ok(Self::Compilation),
            "runtime" => Ok(Self::Runtime),
            "test" => Ok(Self::Test),
            "lint" => Ok(Self::Lint),
            "build" => Ok(Self::Build),
            "config" => Ok(Self::Config),
            "network" => Ok(Self::Network),
            "other" => Ok(Self::Other),
            _ => Err(format!("Unknown error pattern category: {s}")),
        }
    }
}

/// Represents a recurring error pattern detected in a project.
///
/// An error pattern captures the signature, category, and metadata of errors
/// that occur repeatedly, enabling pattern recognition and solution tracking.
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
