//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

crate::define_string_enum! {
    /// Type of execution or command that was run.
    pub enum ExecutionType [strum = "lowercase"] {
        /// Test execution.
        Test,
        /// Linting execution.
        Lint,
        /// Build execution.
        Build,
        /// Continuous integration execution.
        CI,
    }
}

/// Metadata about an execution event (test, lint, build, or CI).
#[skip_serializing_none]
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
