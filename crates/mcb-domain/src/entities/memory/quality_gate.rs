use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityGateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

impl QualityGateStatus {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    #[serde(default)]
    pub id: String,
    pub gate_name: String,
    pub status: QualityGateStatus,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub execution_id: Option<String>,
}
