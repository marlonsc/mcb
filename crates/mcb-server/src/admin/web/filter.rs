//! Query parameter types for server-side entity list filtering, sorting, and pagination.

use rocket::form::FromForm;
use serde::Serialize;

/// Column sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SortOrder {
    /// Ascending (A→Z, 0→9).
    Asc,
    /// Descending (Z→A, 9→0).
    Desc,
}

impl<'v> rocket::form::FromFormField<'v> for SortOrder {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        match field.value.to_lowercase().as_str() {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            _ => Ok(SortOrder::Asc),
        }
    }
}

/// Query-string parameters for entity list pages.
///
/// Parsed by Rocket from `?q=&sort=&order=&page=&per_page=&parent_field=&parent_id=&date_from=&date_to=`.
#[derive(Debug, Clone, FromForm, Serialize)]
pub struct FilterParams {
    /// FK field name to scope by (e.g. `"team_id"`).
    pub parent_field: Option<String>,
    /// Value of the parent FK (e.g. the team's UUID).
    pub parent_id: Option<String>,
    /// Full-text search term.
    #[field(name = "q")]
    pub search: Option<String>,
    /// Column to sort by (validated against `AdminFieldMeta` names).
    #[field(name = "sort")]
    pub sort_field: Option<String>,
    /// Sort direction.
    #[field(name = "order")]
    pub sort_order: Option<SortOrder>,
    /// 1-based page number.
    #[field(default = 1)]
    pub page: usize,
    /// Records per page.
    #[field(default = 20)]
    pub per_page: usize,
    /// ISO date string lower bound for timestamp filtering (e.g. "2026-01-15").
    pub date_from: Option<String>,
    /// ISO date string upper bound for timestamp filtering (e.g. "2026-02-11").
    pub date_to: Option<String>,
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
