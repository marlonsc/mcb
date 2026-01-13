//! HTML response generation helpers
//!
//! Provides reusable HTML fragments for common UI patterns in admin responses.
//! These helpers ensure consistent styling and markup across the admin interface.

use axum::response::Html;

/// Generate an HTML error message with red styling
///
/// # Example
///
/// ```rust,ignore
/// html_error("Failed to update configuration")
/// ```
///
/// Outputs:
/// ```html
/// <div class="text-red-600 dark:text-red-400 mt-2">Error: Failed to update configuration</div>
/// ```
#[inline]
pub fn html_error(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
        message.as_ref()
    ))
}

/// Generate an HTML success message with green styling
///
/// # Example
///
/// ```rust,ignore
/// html_success("Configuration updated successfully")
/// ```
///
/// Outputs:
/// ```html
/// <div class="text-green-600 dark:text-green-400 mt-2">Configuration updated successfully</div>
/// ```
#[inline]
pub fn html_success(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
        message.as_ref()
    ))
}

/// Generate an HTML warning message with yellow styling
///
/// # Example
///
/// ```rust,ignore
/// html_warning("This operation may take some time")
/// ```
///
/// Outputs:
/// ```html
/// <div class="text-yellow-600 dark:text-yellow-400 mt-2">This operation may take some time</div>
/// ```
#[inline]
pub fn html_warning(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-yellow-600 dark:text-yellow-400 mt-2">{}</div>"#,
        message.as_ref()
    ))
}

/// Generate an HTML info message with blue styling
///
/// # Example
///
/// ```rust,ignore
/// html_info("No providers found. Click 'Add Provider' to create one.")
/// ```
///
/// Outputs:
/// ```html
/// <div class="text-blue-600 dark:text-blue-400 mt-2">No providers found. Click 'Add Provider' to create one.</div>
/// ```
#[inline]
pub fn html_info(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-blue-600 dark:text-blue-400 mt-2">{}</div>"#,
        message.as_ref()
    ))
}

// =============================================================================
// HTMX-Specific Helpers - For AJAX responses (used in HTMX handlers)
// =============================================================================

/// Generate HTMX-compatible error message HTML
/// Uses Tailwind classes with dark mode support.
///
/// # Example
/// ```rust,ignore
/// htmx_error("Failed to load metrics")
/// ```
///
/// Outputs:
/// ```html
/// <div class="text-red-500 dark:text-red-400">Failed to load metrics</div>
/// ```
#[inline]
pub fn htmx_error(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-red-500 dark:text-red-400">{}</div>"#,
        message.as_ref()
    ))
}

/// Generate HTMX-compatible loading message HTML
/// Uses gray color to indicate loading state.
///
/// # Example
/// ```rust,ignore
/// htmx_loading()
/// ```
#[inline]
pub fn htmx_loading() -> Html<String> {
    Html(r#"<div class="text-gray-500 dark:text-gray-400">Loading...</div>"#.to_string())
}

/// Generate HTMX-compatible success message HTML
/// Uses green color for success state.
///
/// # Example
/// ```rust,ignore
/// htmx_success("Metrics updated successfully")
/// ```
#[inline]
pub fn htmx_success(message: impl AsRef<str>) -> Html<String> {
    Html(format!(
        r#"<div class="text-green-500 dark:text-green-400">{}</div>"#,
        message.as_ref()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_error() {
        let result = html_error("test error");
        assert!(result.0.contains("text-red-600"));
        assert!(result.0.contains("Error: test error"));
    }

    #[test]
    fn test_html_success() {
        let result = html_success("test success");
        assert!(result.0.contains("text-green-600"));
        assert!(result.0.contains("test success"));
    }

    #[test]
    fn test_html_warning() {
        let result = html_warning("test warning");
        assert!(result.0.contains("text-yellow-600"));
        assert!(result.0.contains("test warning"));
    }

    #[test]
    fn test_html_info() {
        let result = html_info("test info");
        assert!(result.0.contains("text-blue-600"));
        assert!(result.0.contains("test info"));
    }

    #[test]
    fn test_htmx_error() {
        let result = htmx_error("Failed to load metrics");
        assert!(result.0.contains("text-red-500"));
        assert!(result.0.contains("dark:text-red-400"));
        assert!(result.0.contains("Failed to load metrics"));
    }

    #[test]
    fn test_htmx_loading() {
        let result = htmx_loading();
        assert!(result.0.contains("text-gray-500"));
        assert!(result.0.contains("dark:text-gray-400"));
        assert!(result.0.contains("Loading..."));
    }

    #[test]
    fn test_htmx_success() {
        let result = htmx_success("Updated successfully");
        assert!(result.0.contains("text-green-500"));
        assert!(result.0.contains("dark:text-green-400"));
        assert!(result.0.contains("Updated successfully"));
    }
}
