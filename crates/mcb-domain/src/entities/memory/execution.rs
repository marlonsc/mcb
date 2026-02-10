use serde::{Deserialize, Serialize};

/// Type of execution or command that was run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionType {
    /// Test execution.
    Test,
    /// Linting execution.
    Lint,
    /// Build execution.
    Build,
    /// Continuous integration execution.
    CI,
}

impl ExecutionType {
    /// Returns the string representation of the execution type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Lint => "lint",
            Self::Build => "build",
            Self::CI => "ci",
        }
    }
}

impl std::str::FromStr for ExecutionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "test" => Ok(Self::Test),
            "lint" => Ok(Self::Lint),
            "build" => Ok(Self::Build),
            "ci" => Ok(Self::CI),
            _ => Err(format!("Unknown execution type: {s}")),
        }
    }
}

/// Metadata about an execution event (test, lint, build, or CI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Unique identifier for the execution event.
    #[serde(default)]
    pub id: String,
    /// The command that was executed.
    pub command: String,
    /// Exit code returned by the execution (if applicable).
    pub exit_code: Option<i32>,
    /// Duration of the execution in milliseconds.
    pub duration_ms: Option<i64>,
    /// Whether the execution completed successfully.
    pub success: bool,
    /// Type of execution (test, lint, build, or CI).
    pub execution_type: ExecutionType,
    /// Code coverage percentage (if applicable).
    pub coverage: Option<f32>,
    /// List of files affected by the execution.
    pub files_affected: Vec<String>,
    /// Summary of the execution output.
    pub output_summary: Option<String>,
    /// Number of warnings generated during execution.
    pub warnings_count: Option<i32>,
    /// Number of errors generated during execution.
    pub errors_count: Option<i32>,
}
