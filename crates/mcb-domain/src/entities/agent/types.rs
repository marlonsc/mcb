use serde::{Deserialize, Serialize};

/// Enumeration of available agent types in the system.
///
/// Each agent type represents a specialized role in the workflow orchestration system:
/// - `Sisyphus`: Primary orchestration agent with full tool access
/// - `Oracle`: Architecture reviewer and decision maker
/// - `Explore`: Codebase exploration and pattern analysis specialist
/// - `Prometheus`: Strategic planning and roadmap creation
/// - `Momus`: Plan and work verification specialist
/// - `Librarian`: External documentation and OSS research specialist
/// - `Metis`: Pre-planning analysis and feasibility checks
/// - `SisyphusJunior`: Focused task executor for delegated work
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Sisyphus agent - primary orchestration agent with full tool access
    Sisyphus,
    /// Oracle agent - architecture reviewer and decision maker
    Oracle,
    /// Explore agent - codebase exploration and pattern analysis specialist
    Explore,
    /// Prometheus agent - strategic planning and roadmap creation
    Prometheus,
    /// Momus agent - plan and work verification specialist
    Momus,
    /// Librarian agent - external documentation and OSS research specialist
    Librarian,
    /// Metis agent - pre-planning analysis and feasibility checks
    Metis,
    /// SisyphusJunior agent - focused task executor for delegated work
    SisyphusJunior,
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
            Self::Prometheus => "prometheus",
            Self::Momus => "momus",
            Self::Librarian => "librarian",
            Self::Metis => "metis",
            Self::SisyphusJunior => "sisyphus-junior",
        }
    }
}

impl_from_str!(AgentType, "Unknown agent type: {}", {
    "sisyphus" => Self::Sisyphus,
    "oracle" => Self::Oracle,
    "explore" => Self::Explore,
    "prometheus" => Self::Prometheus,
    "momus" => Self::Momus,
    "librarian" => Self::Librarian,
    "metis" => Self::Metis,
    "sisyphus-junior" => Self::SisyphusJunior,
    "sisyphus_junior" => Self::SisyphusJunior,
    "junior" => Self::SisyphusJunior,
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

impl_from_str!(AgentSessionStatus, "Unknown agent session status: {}", {
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

impl_from_str!(CheckpointType, "Unknown checkpoint type: {}", {
    "git" => Self::Git,
    "file" => Self::File,
    "config" => Self::Config,
});
