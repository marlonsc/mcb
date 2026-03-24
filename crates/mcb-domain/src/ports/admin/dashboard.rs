//! Dashboard/analytics query ports.

use async_trait::async_trait;

use crate::error::Result;

/// Response DTO for monthly observation counts
#[derive(Debug, Clone)]
pub struct MonthlyCount {
    /// Month identifier (e.g., "2025-02")
    pub month: String,
    /// Number of observations in this month
    pub count: i64,
}

/// Response DTO for daily observation counts
#[derive(Debug, Clone)]
pub struct DailyCount {
    /// Day identifier (e.g., "2025-02-25")
    pub day: String,
    /// Number of observations on this day
    pub count: i64,
}

/// Response DTO for tool call counts
#[derive(Debug, Clone)]
pub struct ToolCallCount {
    /// Name of the tool
    pub tool_name: String,
    /// Number of times this tool was called
    pub count: i64,
}

/// Response DTO for agent session statistics
#[derive(Debug, Clone)]
pub struct AgentSessionStats {
    /// Total number of sessions
    pub total_sessions: i64,
    /// Total number of unique agents
    pub total_agents: i64,
}

/// Port for dashboard/admin queries
///
/// Provides read-only access to aggregated analytics and statistics
/// for dashboard and admin UI consumption.
#[async_trait]
pub trait DashboardQueryPort: Send + Sync {
    /// Get observations aggregated by month.
    async fn get_observations_by_month(&self, limit: usize) -> Result<Vec<MonthlyCount>>;

    /// Get observations aggregated by day.
    async fn get_observations_by_day(&self, limit: usize) -> Result<Vec<DailyCount>>;

    /// Get tool call counts.
    async fn get_tool_call_counts(&self) -> Result<Vec<ToolCallCount>>;

    /// Get agent session statistics.
    async fn get_agent_session_stats(&self) -> Result<AgentSessionStats>;
}
