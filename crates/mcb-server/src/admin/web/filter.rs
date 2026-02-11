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
    /// Unix-epoch lower bound for timestamp filtering.
    pub date_from: Option<i64>,
    /// Unix-epoch upper bound for timestamp filtering.
    pub date_to: Option<i64>,
}

impl FilterParams {
    #[cfg(test)]
    fn test_default() -> Self {
        Self {
            parent_field: None,
            parent_id: None,
            search: None,
            sort_field: None,
            sort_order: None,
            page: 1,
            per_page: 20,
            date_from: None,
            date_to: None,
        }
    }
}

/// Paginated result returned by `EntityCrudAdapter::list_filtered`.
#[derive(Debug, Clone, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_params_default() {
        let params = FilterParams::test_default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
        assert!(params.search.is_none());
        assert!(params.sort_field.is_none());
        assert!(params.parent_field.is_none());
        assert!(params.parent_id.is_none());
        assert!(params.date_from.is_none());
        assert!(params.date_to.is_none());
    }

    #[test]
    fn test_sort_order_values() {
        assert_eq!(SortOrder::Asc, SortOrder::Asc);
        assert_ne!(SortOrder::Asc, SortOrder::Desc);
    }

    #[test]
    fn test_filtered_result_empty() {
        let result = FilteredResult {
            records: vec![],
            total_count: 0,
            page: 1,
            per_page: 20,
            total_pages: 0,
        };
        assert_eq!(result.total_count, 0);
        assert_eq!(result.total_pages, 0);
    }
}
