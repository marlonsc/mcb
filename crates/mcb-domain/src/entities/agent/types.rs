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

crate::impl_from_str!(AgentType, "Unknown agent type: {}", {
    "sisyphus" => Self::Sisyphus,
    "oracle" => Self::Oracle,
    "explore" => Self::Explore,
});

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

crate::impl_from_str!(AgentSessionStatus, "Unknown agent session status: {}", {
    "active" => Self::Active,
    "completed" => Self::Completed,
    "failed" => Self::Failed,
});

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

crate::impl_from_str!(CheckpointType, "Unknown checkpoint type: {}", {
    "git" => Self::Git,
    "file" => Self::File,
    "config" => Self::Config,
});
