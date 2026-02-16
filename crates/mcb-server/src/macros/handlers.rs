//! Handler utility macros for field extraction and validation.
//!
//! Used by `handlers/memory/` and `handlers/entities/` to reduce
//! repetitive match + early-return boilerplate.

/// Extract a value from a `Result<T, CallToolResult>`, returning the error as `Ok(err)`.
///
/// The `require_*` helpers return `Result<T, CallToolResult>` where the `Err` variant
/// is a tool-level error (not a protocol error). This macro converts the match:
///
/// ```ignore
/// let value = match require_str(data, "field") {
///     Ok(v) => v,
///     Err(error_result) => return Ok(error_result),
/// };
/// ```
///
/// Into a single expression:
///
/// ```ignore
/// let value = extract_field!(require_str(data, "field"));
/// ```
macro_rules! extract_field {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error_result) => return Ok(error_result),
        }
    };
}

/// Parse a string into an enum via `FromStr`, returning a tool error on failure.
///
/// Replaces the common pattern:
///
/// ```ignore
/// let typed: MyEnum = match string_val.parse() {
///     Ok(v) => v,
///     Err(_) => return Ok(tool_error(format!("Invalid field: {string_val}"))),
/// };
/// ```
///
/// With:
///
/// ```ignore
/// let typed: MyEnum = parse_enum!(string_val, "field");
/// ```
macro_rules! parse_enum {
    ($str_val:expr, $label:expr) => {
        match $str_val.parse() {
            Ok(v) => v,
            Err(_) => {
                return Ok($crate::utils::mcp::tool_error(format!(
                    "Invalid {}: {}",
                    $label, $str_val
                )));
            }
        }
    };
}

/// Extract a required argument from an `Option<String>`, returning `McpError` on `None`.
///
/// Replaces the repeated pattern in entity handlers:
///
/// ```ignore
/// let project_id = args.project_id.as_deref().ok_or_else(|| {
///     McpError::invalid_params("project_id required for list", None)
/// })?;
/// ```
///
/// With:
///
/// ```ignore
/// let project_id = require_arg!(args.project_id, "project_id required for list");
/// ```
macro_rules! require_arg {
    ($opt:expr, $msg:literal) => {
        $opt.as_deref()
            .ok_or_else(|| McpError::invalid_params($msg, None))?
    };
}
