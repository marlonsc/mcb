//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! Response formatting utilities for MCP server.

mod indexing;
mod search;
mod validation;

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus, ValidationReport};
use mcb_domain::value_objects::SearchResult;
use mcb_domain::{error, info};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

use crate::error_mapping::safe_internal_error;

/// Response formatter for MCP server tools.
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format search response for display.
    ///
    /// # Errors
    /// Returns an error if response content serialization fails.
    pub fn format_search_response(
        query: &str,
        results: &[SearchResult],
        duration: Duration,
        limit: usize,
    ) -> Result<CallToolResult, McpError> {
        let message = search::build_search_response_message(query, results, duration, limit);
        info!(
            "ResponseFormatter",
            "search completed",
            &format!(
                "results={} duration={:?} limit={limit}",
                results.len(),
                duration
            )
        );
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    /// Format indexing success response.
    #[must_use]
    pub fn format_indexing_success(
        result: &IndexingResult,
        path: &Path,
        duration: Duration,
    ) -> CallToolResult {
        let message = indexing::build_indexing_success_message(result, path, duration);
        info!(
            "ResponseFormatter",
            "indexing completed",
            &format!("chunks={} duration={:?}", result.chunks_created, duration)
        );
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format indexing error response.
    #[must_use]
    pub fn format_indexing_error(error: &str, path: &Path) -> CallToolResult {
        let message = indexing::build_indexing_error_message(error, path);
        let detail = format!("path={} error={error}", path.display());
        error!("ResponseFormatter", "indexing failed", &detail);
        CallToolResult::error(vec![Content::text(message)])
    }

    /// Format indexing status response.
    #[must_use]
    pub fn format_indexing_status(status: &IndexingStatus) -> CallToolResult {
        let message = indexing::build_indexing_status_message(status);
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Serializes a value into pretty JSON and wraps it in a successful MCP tool result.
    ///
    /// # Errors
    /// Returns an error when JSON serialization fails.
    pub fn json_success<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
        let json = serde_json::to_string_pretty(value)
            .map_err(|e| safe_internal_error("json serialization", &e))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Format clear index response.
    #[must_use]
    pub fn format_clear_index(collection: &str) -> CallToolResult {
        let message = format!(
            "✅ **Index Cleared**\n\n\
             Collection `{collection}` has been cleared.\n\
             Run `index_repo` to rebuild the index."
        );
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format validation success response.
    #[must_use]
    pub fn format_validation_success(
        report: &ValidationReport,
        path: &Path,
        duration: Duration,
    ) -> CallToolResult {
        let message = validation::build_validation_message(report, path, duration);
        info!(
            "ResponseFormatter",
            "validation completed",
            &format!(
                "violations={} duration={:?}",
                report.total_violations, duration
            )
        );
        if report.passed {
            CallToolResult::success(vec![Content::text(message)])
        } else {
            CallToolResult::error(vec![Content::text(message)])
        }
    }

    /// Format validation error response.
    #[must_use]
    pub fn format_validation_error(error: &str, path: &Path) -> CallToolResult {
        let message = validation::build_validation_error_message(error, path);
        let detail = format!("path={} error={error}", path.display());
        error!("ResponseFormatter", "validation failed", &detail);
        CallToolResult::error(vec![Content::text(message)])
    }
}
