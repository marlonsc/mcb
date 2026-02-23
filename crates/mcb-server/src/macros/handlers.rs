//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
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

/// Extract a required `Option` field from `AdminState`, returning
/// `AdminError::unavailable` when `None`.
///
/// ```ignore
/// let cache = require_service!(state, cache, "Cache provider not available");
/// ```
macro_rules! require_service {
    ($state:expr, $field:ident, $msg:literal) => {
        match $state.$field {
            Some(ref svc) => svc,
            None => return Err($crate::admin::error::AdminError::unavailable($msg)),
        }
    };
}

/// Generate a simple template page handler that renders a Handlebars template
/// with standard nav context.
///
/// ```ignore
/// template_page!(config_page, "admin/config", "Configuration", "config");
/// ```
macro_rules! template_page {
    ($fn_name:ident, $template:literal, $title:literal, $page:literal) => {
        #[allow(missing_docs)]
        pub async fn $fn_name() -> $crate::templates::Template {
            mcb_domain::info!("web", concat!(stringify!($fn_name), " called"));
            $crate::templates::Template::render(
                $template,
                context! {
                    title: $title,
                    current_page: $page,
                    nav_groups: $crate::admin::web::view_model::nav_groups(),
                },
            )
        }
    };
}

macro_rules! template_page_with_path {
    (
        $fn_name:ident,
        $param:ident : $param_ty:ty,
        $template:literal,
        $title:literal,
        $current_page:literal
    ) => {
        #[allow(missing_docs)]
        pub async fn $fn_name(
            axum::extract::Path($param): axum::extract::Path<$param_ty>,
        ) -> $crate::templates::Template {
            let _ = $param;
            mcb_domain::info!("web", concat!(stringify!($fn_name), " called"));
            $crate::templates::Template::render(
                $template,
                context! {
                    title: $title,
                    current_page: $current_page,
                    nav_groups: $crate::admin::web::view_model::nav_groups(),
                },
            )
        }
    };
}

/// Generate a handler constructor that extracts dependencies from `AppContext`.
///
/// This macro eliminates boilerplate `pub fn new(...)` constructors by automatically
/// generating them from a list of field names and their types.
///
/// # Example
///
/// ```ignore
/// handler_new!(MyHandler {
///     repo: Arc<dyn MyRepository>,
///     service: Arc<dyn MyService>,
/// });
/// ```
///
/// Expands to:
///
/// ```ignore
/// impl MyHandler {
///     pub fn new(repo: Arc<dyn MyRepository>, service: Arc<dyn MyService>) -> Self {
///         Self {
///             repo,
///             service,
///         }
///     }
/// }
/// ```
macro_rules! handler_new {
    (
        $handler_name:ident {
            $($field:ident: $field_ty:ty),* $(,)?
        }
    ) => {
        impl $handler_name {
            /// Create a new handler with the given dependencies.
            pub fn new($($field: $field_ty),*) -> Self {
                Self {
                    $($field),*
                }
            }
        }
    };
}
