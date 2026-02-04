use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Sisyphus,
    Oracle,
    Explore,
}

impl AgentType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sisyphus => "sisyphus",
            Self::Oracle => "oracle",
            Self::Explore => "explore",
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sisyphus" => Ok(Self::Sisyphus),
            "oracle" => Ok(Self::Oracle),
            "explore" => Ok(Self::Explore),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSessionStatus {
    Active,
    Completed,
    Failed,
}

impl AgentSessionStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for AgentSessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown agent session status: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointType {
    Git,
    File,
    Config,
}

impl CheckpointType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::File => "file",
            Self::Config => "config",
        }
    }
}

impl std::str::FromStr for CheckpointType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "file" => Ok(Self::File),
            "config" => Ok(Self::Config),
            _ => Err(format!("Unknown checkpoint type: {s}")),
        }
    }
}
