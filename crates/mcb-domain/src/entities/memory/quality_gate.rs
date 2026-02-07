use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the status of a quality gate check.
///
/// Quality gates are validation checks that can result in one of four states:
/// - `Passed`: The check completed successfully and met all criteria
/// - `Failed`: The check completed but did not meet required criteria
/// - `Warning`: The check completed with non-critical issues
/// - `Skipped`: The check was not executed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityGateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

impl QualityGateStatus {
    /// Converts the quality gate status to its string representation.
    ///
    /// Returns a static string slice representing the status value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Warning => "warning",
            Self::Skipped => "skipped",
        }
    }
}

impl std::str::FromStr for QualityGateStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "warning" => Ok(Self::Warning),
            "skipped" => Ok(Self::Skipped),
            _ => Err(format!("Unknown quality gate status: {s}")),
        }
    }
}

/// Represents the result of a quality gate execution.
///
/// Contains the outcome and metadata of a single quality gate check, including
/// the gate name, status, optional message, and timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    /// Unique identifier for this quality gate result.
    #[serde(default)]
    pub id: String,
    /// Name of the quality gate that was executed.
    pub gate_name: String,
    /// The status outcome of the quality gate check.
    pub status: QualityGateStatus,
    /// Optional message providing additional details about the result.
    pub message: Option<String>,
    /// Timestamp when the quality gate was executed.
    pub timestamp: DateTime<Utc>,
    /// Optional identifier linking this result to a specific execution context.
    pub execution_id: Option<String>,
}
