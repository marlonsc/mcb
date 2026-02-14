use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents the status of a quality gate check.
///
/// Quality gates are validation checks that can result in one of four states:
/// - `Passed`: The check completed successfully and met all criteria
/// - `Failed`: The check completed but did not meet required criteria
/// - `Warning`: The check completed with non-critical issues
/// - `Skipped`: The check was not executed
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
pub enum QualityGateStatus {
    /// Represents the Passed variant.
    Passed,
    /// Represents the Failed variant.
    Failed,
    /// Represents the Warning variant.
    Warning,
    /// Represents the Skipped variant.
    Skipped,
}

impl QualityGateStatus {
    /// Converts the quality gate status to its string representation.
    ///
    /// Returns a static string slice representing the status value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// Represents the result of a quality gate execution.
///
/// Contains the outcome and metadata of a single quality gate check, including
/// the gate name, status, optional message, and timing information.
#[skip_serializing_none]
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
    /// Stores the timestamp value.
    pub timestamp: i64,
    /// Optional identifier linking this result to a specific execution context.
    pub execution_id: Option<String>,
}
