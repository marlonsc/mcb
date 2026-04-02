//! Indexing result formatting.

use std::fmt::Write;
use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus};

pub(super) fn build_indexing_success_message(
    result: &IndexingResult,
    path: &Path,
    duration: Duration,
) -> String {
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
        let _ = write!(
            message,
            "\n⚠️ **Errors encountered:** {}\n",
            result.errors.len()
        );
        for error in &result.errors {
            let _ = writeln!(message, "• {error}");
        }
    } else {
        message.push_str("\n🎯 **Next Steps:**\n");
        message.push_str("• Use `search` with resource=code for semantic queries\n");
        message.push_str(
            "• Try queries like \"find authentication functions\" or \"show error handling\"\n",
        );
    }

    message
}

fn build_indexing_started_message(result: &IndexingResult, path: &Path) -> String {
    let operation_id = result.operation_id.as_ref().map_or_else(
        || "N/A".to_owned(),
        mcb_domain::value_objects::OperationId::as_str,
    );

    format!(
        "🚀 **Indexing Started**\n\n\
         📁 **Path:** `{}`\n\
         🔑 **Operation ID:** `{}`\n\
         📊 **Status:** {}\n\n\
         💡 **Note:** Indexing is running in the background.\n\
         Use `index` action=status to check progress.\n\
         Once complete, use `search` (resource=code) to query the index.",
        path.display(),
        operation_id,
        result.status
    )
}

pub(super) fn build_indexing_error_message(error: &str, path: &Path) -> String {
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

pub(super) fn build_indexing_status_message(status: &IndexingStatus) -> String {
    let mut message = String::new();

    if status.is_indexing {
        message.push_str("🔄 **Indexing Status: In Progress**\n");
        let _ = writeln!(message, "Progress: {:.1}%", status.progress * 100.0);
        if let Some(current_file) = &status.current_file {
            let _ = writeln!(message, "Current file: `{current_file}`");
        }
        let _ = writeln!(
            message,
            "Files processed: {}/{}",
            status.processed_files, status.total_files
        );
    } else {
        message.push_str("📋 **Indexing Status: Idle**\n");
        if status.total_files > 0 {
            let _ = writeln!(
                message,
                "Last run processed {}/{} files",
                status.processed_files, status.total_files
            );
        } else {
            message.push_str("No indexing operation is currently running.\n");
        }
    }

    message
}
