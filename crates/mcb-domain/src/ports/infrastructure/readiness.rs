//! Runtime readiness contracts.

use serde::Serialize;

/// Readiness state for one required runtime dependency.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReadinessDependency {
    /// Stable dependency identifier.
    pub name: &'static str,
    /// Whether the dependency is currently usable.
    pub ready: bool,
    /// Typed failure detail when the dependency is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Complete readiness result for the server.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReadinessReport {
    /// True only when every required dependency is usable.
    pub ready: bool,
    /// Individual dependency results.
    pub dependencies: Vec<ReadinessDependency>,
}

/// Checks dependencies that gate admission of application traffic.
#[async_trait::async_trait]
pub trait ReadinessProvider: Send + Sync {
    /// Check database, migrations, embedding, and vector-store usability.
    async fn check(&self) -> ReadinessReport;
}
