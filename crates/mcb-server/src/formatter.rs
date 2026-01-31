//! Response formatting utilities for MCP server
//!
//! This module contains utilities for formatting tool responses in a consistent,
//! user-friendly way. It handles the presentation of search results, indexing status,
//! and error messages.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::path::Path;
use std::time::Duration;

use mcb_application::domain_services::search::{IndexingResult, IndexingStatus};
use mcb_application::ports::services::ValidationReport;
use mcb_domain::SearchResult;

/// Response formatter for MCP server tools
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format search response for display
    pub fn format_search_response(
        query: &str,
        results: &[SearchResult],
        duration: Duration,
        limit: usize,
    ) -> Result<CallToolResult, McpError> {
        let message = build_search_response_message(query, results, duration, limit);
        tracing::info!(
            "Search completed: found {} results in {:?}",
            results.len(),
            duration
        );
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    /// Format indexing completion response
    pub fn format_indexing_success(
        result: &IndexingResult,
        path: &Path,
        duration: Duration,
    ) -> CallToolResult {
        let message = build_indexing_success_message(result, path, duration);
        tracing::info!(
            "Indexing completed successfully: {} chunks in {:?}",
            result.chunks_created,
            duration
        );
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format indexing error response
    pub fn format_indexing_error(error: &str, path: &Path) -> CallToolResult {
        let message = build_indexing_error_message(error, path);
        tracing::error!("Indexing failed for path {}: {}", path.display(), error);
        CallToolResult::error(vec![Content::text(message)])
    }

    /// Format indexing status response
    pub fn format_indexing_status(status: &IndexingStatus) -> CallToolResult {
        let message = build_indexing_status_message(status);
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format clear index response
    pub fn format_clear_index(collection: &str) -> CallToolResult {
        let message = format!(
            "✅ **Index Cleared**\n\nCollection `{}` has been cleared successfully.",
            collection
        );
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format validation success response
    pub fn format_validation_success(
        report: &ValidationReport,
        path: &Path,
        duration: Duration,
    ) -> CallToolResult {
        let message = build_validation_message(report, path, duration);
        tracing::info!(
            "Validation completed: {} violations in {:?}",
            report.total_violations,
            duration
        );
        if report.passed {
            CallToolResult::success(vec![Content::text(message)])
        } else {
            CallToolResult::error(vec![Content::text(message)])
        }
    }

    /// Format validation error response
    pub fn format_validation_error(error: &str, path: &Path) -> CallToolResult {
        let message = build_validation_error_message(error, path);
        tracing::error!("Validation failed for path {}: {}", path.display(), error);
        CallToolResult::error(vec![Content::text(message)])
    }
}

// ============================================================================
// Search Formatting Functions
// ============================================================================

fn build_search_response_message(
    query: &str,
    results: &[SearchResult],
    duration: Duration,
    limit: usize,
) -> String {
    let mut message = "🔍 **Semantic Code Search Results**\n\n".to_string();
    message.push_str(&format!("**Query:** \"{}\" \n", query));
    message.push_str(&format!(
        "**Search completed in:** {:.2}s\n",
        duration.as_secs_f64()
    ));
    message.push_str(&format!("**Results found:** {}\n\n", results.len()));

    if results.is_empty() {
        append_empty_search_response(&mut message);
    } else {
        append_search_results(&mut message, results, limit, duration);
    }

    message
}

fn append_empty_search_response(message: &mut String) {
    message.push_str("❌ **No Results Found**\n\n");
    message.push_str("**Possible Reasons:**\n");
    message.push_str("• Codebase not indexed yet (run `index_codebase` first)\n");
    message.push_str("• Query terms not present in the codebase\n");
    message.push_str("• Try different keywords or more general terms\n\n");
    message.push_str("**Search Tips:**\n");
    message.push_str("• Use natural language: \"find error handling\", \"authentication logic\"\n");
    message.push_str("• Be specific: \"HTTP request middleware\" > \"middleware\"\n");
    message.push_str("• Include technologies: \"React component state management\"\n");
    message.push_str("• Try synonyms: \"validate\" instead of \"check\"\n");
}

fn append_search_results(
    message: &mut String,
    results: &[SearchResult],
    limit: usize,
    duration: Duration,
) {
    message.push_str("📊 **Search Results:**\n\n");

    for (i, result) in results.iter().enumerate() {
        message.push_str(&format!(
            "**{}.** 📁 `{}` (line {})\n",
            i + 1,
            result.file_path,
            result.start_line
        ));

        append_code_preview(message, result);
        message.push_str(&format!("🎯 **Relevance Score:** {:.3}\n\n", result.score));
    }

    if results.len() == limit {
        message.push_str(&format!(
            "💡 **Showing top {} results.** For more results, try:\n",
            limit
        ));
        message.push_str("• More specific search terms\n");
        message.push_str("• Different query formulations\n");
        message.push_str("• Breaking complex queries into simpler ones\n");
    }

    if duration.as_millis() > 1000 {
        message.push_str(&format!(
            "\n⚠️ **Performance Note:** Search took {:.2}s. \
            Consider using more specific queries for faster results.\n",
            duration.as_secs_f64()
        ));
    }
}

fn append_code_preview(message: &mut String, result: &SearchResult) {
    let lines: Vec<&str> = result.content.lines().collect();
    let preview_lines = if lines.len() > 10 {
        lines
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        result.content.clone()
    };

    let file_ext = Path::new(&result.file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let lang_hint = get_language_hint(file_ext, &result.language);

    if lang_hint.is_empty() {
        message.push_str(&format!("```\n{}\n```\n", preview_lines));
    } else {
        message.push_str(&format!("``` {}\n{}\n```\n", lang_hint, preview_lines));
    }
}

fn get_language_hint<'a>(file_ext: &str, default_lang: &'a str) -> &'a str {
    match file_ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "go" => "go",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "cs" => "csharp",
        _ => default_lang,
    }
}

// ============================================================================
// Indexing Formatting Functions
// ============================================================================

fn build_indexing_success_message(
    result: &IndexingResult,
    path: &Path,
    duration: Duration,
) -> String {
    // Check if this is an async "started" response
    if result.status == "started" {
        return build_indexing_started_message(result, path);
    }

    let duration_secs = duration.as_secs_f64();
    let chunks_per_sec = if duration_secs > 0.0 {
        result.chunks_created as f64 / duration_secs
    } else {
        result.chunks_created as f64
    };

    let mut message = format!(
        "✅ **Indexing Completed Successfully**\n\n\
         📊 **Statistics**:\n\
         • Files processed: {}\n\
         • Chunks created: {}\n\
         • Files skipped: {}\n\
         • Source directory: `{}`\n\
         • Processing time: {:.2}s\n\
         • Performance: {:.0} chunks/sec\n",
        result.files_processed,
        result.chunks_created,
        result.files_skipped,
        path.display(),
        duration_secs,
        chunks_per_sec
    );

    if !result.errors.is_empty() {
        message.push_str(&format!(
            "\n⚠️ **Errors encountered:** {}\n",
            result.errors.len()
        ));
        for error in &result.errors {
            message.push_str(&format!("• {}\n", error));
        }
    } else {
        message.push_str("\n🎯 **Next Steps:**\n");
        message.push_str("• Use `search_code` for semantic queries\n");
        message.push_str(
            "• Try queries like \"find authentication functions\" or \"show error handling\"\n",
        );
    }

    message
}

fn build_indexing_started_message(result: &IndexingResult, path: &Path) -> String {
    let operation_id = result.operation_id.as_deref().unwrap_or("unknown");

    format!(
        "🚀 **Indexing Started**\n\n\
         📁 **Path:** `{}`\n\
         🔑 **Operation ID:** `{}`\n\
         📊 **Status:** {}\n\n\
         💡 **Note:** Indexing is running in the background.\n\
         Use `get_indexing_status` to check progress.\n\
         Once complete, use `search_code` to query the index.",
        path.display(),
        operation_id,
        result.status
    )
}

fn build_indexing_error_message(error: &str, path: &Path) -> String {
    format!(
        "❌ **Indexing Failed**\n\n\
         **Error Details**: {}\n\n\
         **Troubleshooting:**\n\
         • Verify the directory contains readable source files\n\
         • Check file permissions and access rights\n\
         • Ensure supported file types (.rs, .py, .js, .ts, etc.)\n\
         • Try indexing a smaller directory first\n\n\
         **Supported Languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C++, C#\n\n\
         **Path**: `{}`",
        error,
        path.display()
    )
}

fn build_indexing_status_message(status: &IndexingStatus) -> String {
    let mut message = String::new();

    if status.is_indexing {
        message.push_str("🔄 **Indexing Status: In Progress**\n");
        message.push_str(&format!("Progress: {:.1}%\n", status.progress * 100.0));
        if let Some(current_file) = &status.current_file {
            message.push_str(&format!("Current file: `{}`\n", current_file));
        }
        message.push_str(&format!(
            "Files processed: {}/{}\n",
            status.processed_files, status.total_files
        ));
    } else {
        message.push_str("📋 **Indexing Status: Idle**\n");
        if status.total_files > 0 {
            message.push_str(&format!(
                "Last run processed {}/{} files\n",
                status.processed_files, status.total_files
            ));
        } else {
            message.push_str("No indexing operation is currently running.\n");
        }
    }

    message
}

// ============================================================================
// Validation Formatting Functions
// ============================================================================

fn build_validation_message(report: &ValidationReport, path: &Path, duration: Duration) -> String {
    // Return JSON structured output as per plan specification
    let json_output = serde_json::json!({
        "workspace": path.display().to_string(),
        "passed": report.passed,
        "total_violations": report.total_violations,
        "errors": report.errors,
        "warnings": report.warnings,
        "infos": report.infos,
        "duration_secs": duration.as_secs_f64(),
        "violations": report.violations.iter().map(|v| {
            serde_json::json!({
                "id": v.id,
                "category": v.category,
                "severity": v.severity,
                "file": v.file,
                "line": v.line,
                "message": v.message,
                "suggestion": v.suggestion
            })
        }).collect::<Vec<_>>()
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
        format!(
            "{{\"error\": \"Failed to serialize validation report\", \"path\": \"{}\"}}",
            path.display()
        )
    })
}

fn build_validation_error_message(error: &str, path: &Path) -> String {
    // Return JSON structured error output
    let json_output = serde_json::json!({
        "error": error,
        "path": path.display().to_string(),
        "passed": false
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
        format!(
            "{{\"error\": \"{}\", \"path\": \"{}\"}}",
            error,
            path.display()
        )
    })
}
