use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{SqlParam, SqlRow};

/// Helper to get a required string field.
pub fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
    row.try_get_string(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

/// Helper to get a required i64 field.
pub fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
    row.try_get_i64(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

/// Helper to get an optional string field.
pub fn opt_str(row: &dyn SqlRow, col: &str) -> Result<Option<String>> {
    row.try_get_string(col)
}

/// Helper to get an optional i64 field.
pub fn opt_i64(row: &dyn SqlRow, col: &str) -> Result<Option<i64>> {
    row.try_get_i64(col)
}

/// Helper to convert Option<String> to SqlParam.
pub fn opt_str_param(value: &Option<String>) -> SqlParam {
    match value {
        Some(v) => SqlParam::String(v.clone()),
        None => SqlParam::Null,
    }
}

/// Helper to convert Option<i64> to SqlParam.
pub fn opt_i64_param(value: Option<i64>) -> SqlParam {
    match value {
        Some(v) => SqlParam::I64(v),
        None => SqlParam::Null,
    }
}
