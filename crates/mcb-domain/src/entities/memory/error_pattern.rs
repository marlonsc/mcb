use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorPatternCategory {
    Compilation,
    Runtime,
    Test,
    Lint,
    Build,
    Config,
    Network,
    Other,
}

impl ErrorPatternCategory {
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
    type Err = String;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub project_id: String,
    pub pattern_signature: String,
    pub description: String,
    pub category: ErrorPatternCategory,
    pub solutions: Vec<String>,
    pub affected_files: Vec<String>,
    pub tags: Vec<String>,
    pub occurrence_count: i64,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
    pub embedding_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPatternMatch {
    pub id: String,
    pub pattern_id: String,
    pub observation_id: String,
    pub confidence: i64,
    pub solution_applied: Option<i32>,
    pub resolution_successful: Option<bool>,
    pub matched_at: i64,
    pub resolved_at: Option<i64>,
}
