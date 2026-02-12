use serde::{Deserialize, Serialize};

/// Enumeration of available agent types in the system.
///
/// Each agent type represents a specialized role in the workflow orchestration system:
/// - `Sisyphus`: Primary orchestration agent with full tool access
/// - `Oracle`: High-IQ consultation for architecture and complex debugging (read-only)
/// - `Explore`: Codebase exploration and pattern analysis
/// - `Prometheus`: Strategic planning, roadmaps, and task breakdown
/// - `Momus`: Plan and work verification, quality gate
/// - `Librarian`: External documentation and OSS examples research
/// - `Metis`: Pre-planning analysis and feasibility checks
/// - `SisyphusJunior`: Focused task executor (delegated via category)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Sisyphus agent - primary orchestration agent
    Sisyphus,
    /// Oracle agent - architecture reviewer and decision maker
    Oracle,
    /// Explore agent - codebase exploration and pattern analysis
    Explore,
    /// Prometheus agent - strategic planning and task breakdown
    Prometheus,
    /// Momus agent - plan verification and quality gate
    Momus,
    /// Librarian agent - external documentation and OSS research
    Librarian,
    /// Metis agent - pre-planning analysis and feasibility checks
    Metis,
    /// SisyphusJunior agent - focused task executor
    SisyphusJunior,
    /// Hephaestus agent - tooling and infrastructure builder
    Hephaestus,
    /// Atlas agent - codebase mapping and navigation
    Atlas,
    /// MultimodalLooker agent - visual analysis and media inspection
    MultimodalLooker,
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
            Self::Hephaestus => "hephaestus",
            Self::Atlas => "atlas",
            Self::MultimodalLooker => "multimodal-looker",
        }
    }
}

impl_from_str!(AgentType, "Unknown agent type: {}. Valid types: sisyphus, oracle, explore, prometheus, momus, librarian, metis, sisyphus-junior (aliases: junior, sisyphus_junior), hephaestus, atlas, multimodal-looker (aliases: looker, multimodal_looker)", {
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
    "hephaestus" => Self::Hephaestus,
    "atlas" => Self::Atlas,
    "multimodal-looker" => Self::MultimodalLooker,
    "multimodal_looker" => Self::MultimodalLooker,
    "looker" => Self::MultimodalLooker,
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
