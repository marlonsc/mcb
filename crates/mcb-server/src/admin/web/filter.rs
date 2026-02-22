//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Query parameter types for server-side entity list filtering, sorting, and pagination.

use serde::{Deserialize, Serialize};

/// Column sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending (A→Z, 0→9).
    #[default]
    Asc,
    /// Descending (Z→A, 9→0).
    Desc,
}

/// Query-string parameters for entity list pages.
///
/// Parsed from query parameters: `?q=&sort=&order=&page=&per_page=&parent_field=&parent_id=&date_from=&date_to=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterParams {
    /// FK field name to scope by (e.g. `"team_id"`).
    pub parent_field: Option<String>,
    /// Value of the parent FK (e.g. the team's UUID).
    pub parent_id: Option<String>,
    /// Full-text search term (from query param "q").
    #[serde(rename = "q", default)]
    pub search: Option<String>,
    /// Column to sort by (validated against `AdminFieldMeta` names, from query param "sort").
    #[serde(rename = "sort", default)]
    pub sort_field: Option<String>,
    /// Sort direction (from query param "order").
    #[serde(rename = "order", default)]
    pub sort_order: Option<SortOrder>,
    /// 1-based page number (default 1).
    #[serde(default = "default_page")]
    pub page: usize,
    /// Records per page (default 20).
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    /// ISO date string lower bound for timestamp filtering (e.g. "2026-01-15").
    pub date_from: Option<String>,
    /// ISO date string upper bound for timestamp filtering (e.g. "2026-02-11").
    pub date_to: Option<String>,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

/// Parse an ISO date string ("YYYY-MM-DD") to a Unix epoch timestamp (start of day UTC).
/// Returns `None` if the string is empty or unparseable.
#[must_use]
pub fn parse_iso_date_to_epoch(date_str: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
}

/// Parse an ISO date string to end-of-day epoch (23:59:59 UTC).
#[must_use]
pub fn parse_iso_date_to_epoch_end(date_str: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc().timestamp())
}

/// Paginated result returned by `EntityCrudAdapter::list_filtered`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FilteredResult {
    /// Records for the current page.
    pub records: Vec<serde_json::Value>,
    /// Total records matching filter (before pagination).
    pub total_count: usize,
    /// Current 1-based page.
    pub page: usize,
    /// Page size.
    pub per_page: usize,
    /// Computed ceiling of `total_count / per_page`.
    pub total_pages: usize,
}
