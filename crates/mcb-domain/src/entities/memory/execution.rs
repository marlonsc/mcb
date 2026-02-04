use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionType {
    Test,
    Lint,
    Build,
    CI,
}

impl ExecutionType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    #[serde(default)]
    pub id: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub success: bool,
    pub execution_type: ExecutionType,
    pub coverage: Option<f32>,
    pub files_affected: Vec<String>,
    pub output_summary: Option<String>,
    pub warnings_count: Option<i32>,
    pub errors_count: Option<i32>,
}
