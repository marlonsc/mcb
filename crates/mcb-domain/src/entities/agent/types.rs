use serde::{Deserialize, Serialize};

/// Enumeration of available agent types in the system.
///
/// Each agent type represents a specialized role in the workflow orchestration system:
/// - `Sisyphus`: Focused executor for task implementation
/// - `Oracle`: Architecture reviewer and decision maker
/// - `Explore`: Research and investigation specialist
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Sisyphus agent - focused executor for implementing tasks
    Sisyphus,
    /// Oracle agent - architecture reviewer and decision maker
    Oracle,
    /// Explore agent - research and investigation specialist
    Explore,
}

impl AgentType {
    /// Converts the agent type to its string representation.
    ///
    /// # Returns
    /// A static string slice representing the agent type in lowercase.
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

    /// Parses a string into an `AgentType`.
    ///
    /// The parsing is case-insensitive. Valid inputs are "sisyphus", "oracle", and "explore".
    ///
    /// # Errors
    /// Returns an error if the string does not match any known agent type.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sisyphus" => Ok(Self::Sisyphus),
            "oracle" => Ok(Self::Oracle),
            "explore" => Ok(Self::Explore),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

/// Enumeration of possible states for an agent session.
///
/// Represents the lifecycle of an agent session from creation through completion or failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSessionStatus {
    /// Session is currently active and processing
    Active,
    /// Session has completed successfully
    Completed,
    /// Session has failed
    Failed,
}

impl AgentSessionStatus {
    /// Converts the session status to its string representation.
    ///
    /// # Returns
    /// A static string slice representing the status in lowercase.
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

    /// Parses a string into an `AgentSessionStatus`.
    ///
    /// The parsing is case-insensitive. Valid inputs are "active", "completed", and "failed".
    ///
    /// # Errors
    /// Returns an error if the string does not match any known session status.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown agent session status: {s}")),
        }
    }
}

/// Enumeration of checkpoint types for session state persistence.
///
/// Represents different mechanisms for saving and restoring session state during execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointType {
    /// Git-based checkpoint - state saved in version control
    Git,
    /// File-based checkpoint - state saved to filesystem
    File,
    /// Configuration-based checkpoint - state saved in config
    Config,
}

impl CheckpointType {
    /// Converts the checkpoint type to its string representation.
    ///
    /// # Returns
    /// A static string slice representing the checkpoint type in lowercase.
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

    /// Parses a string into a `CheckpointType`.
    ///
    /// The parsing is case-insensitive. Valid inputs are "git", "file", and "config".
    ///
    /// # Errors
    /// Returns an error if the string does not match any known checkpoint type.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "file" => Ok(Self::File),
            "config" => Ok(Self::Config),
            _ => Err(format!("Unknown checkpoint type: {s}")),
        }
    }
}
