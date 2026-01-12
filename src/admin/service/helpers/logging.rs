//! Logging operations helper module
//!
//! Provides functions for log retrieval, filtering, export, and statistics.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{
    AdminError, LogEntries, LogEntry, LogExportFormat, LogFilter, LogStats,
};
use crate::infrastructure::logging::SharedLogBuffer;
use std::collections::HashMap;

/// Get filtered log entries from the log buffer
pub async fn get_logs(
    log_buffer: &SharedLogBuffer,
    filter: LogFilter,
) -> Result<LogEntries, AdminError> {
    let core_entries = log_buffer.get_all().await;

    let mut entries: Vec<LogEntry> = core_entries
        .into_iter()
        .map(|e| LogEntry {
            timestamp: e.timestamp,
            level: e.level,
            module: e.target.clone(),
            message: e.message,
            target: e.target,
            file: None,
            line: None,
        })
        .collect();

    // Apply filters
    if let Some(level) = filter.level {
        entries.retain(|e| e.level == level);
    }
    if let Some(module) = filter.module {
        entries.retain(|e| e.module == module);
    }
    if let Some(message_contains) = filter.message_contains {
        entries.retain(|e| e.message.contains(&message_contains));
    }
    if let Some(start_time) = filter.start_time {
        entries.retain(|e| e.timestamp >= start_time);
    }
    if let Some(end_time) = filter.end_time {
        entries.retain(|e| e.timestamp <= end_time);
    }

    let total_count = entries.len() as u64;

    if let Some(limit) = filter.limit {
        entries.truncate(limit);
    }

    Ok(LogEntries {
        entries,
        total_count,
        has_more: false,
    })
}

/// Export logs to a file in the specified format
pub async fn export_logs(
    log_buffer: &SharedLogBuffer,
    filter: LogFilter,
    format: LogExportFormat,
) -> Result<String, AdminError> {
    let log_entries = get_logs(log_buffer, filter).await?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let extension = match format {
        LogExportFormat::Json => "json",
        LogExportFormat::Csv => "csv",
        LogExportFormat::PlainText => "log",
    };

    let export_dir = std::path::PathBuf::from(admin_defaults::DEFAULT_EXPORTS_DIR);
    std::fs::create_dir_all(&export_dir).map_err(|e| {
        AdminError::ConfigError(format!("Failed to create exports directory: {}", e))
    })?;

    let filename = format!("logs_export_{}.{}", timestamp, extension);
    let filepath = export_dir.join(&filename);

    let content = match format {
        LogExportFormat::Json => serde_json::to_string_pretty(&log_entries.entries)
            .map_err(|e| AdminError::ConfigError(format!("JSON serialization failed: {}", e)))?,
        LogExportFormat::Csv => {
            let mut csv = String::from("timestamp,level,module,target,message\n");
            for entry in &log_entries.entries {
                csv.push_str(&format!(
                    "{},{},{},{},\"{}\"\n",
                    entry.timestamp.to_rfc3339(),
                    entry.level,
                    entry.module,
                    entry.target,
                    entry.message.replace('"', "\"\"")
                ));
            }
            csv
        }
        LogExportFormat::PlainText => {
            let mut text = String::new();
            for entry in &log_entries.entries {
                text.push_str(&format!(
                    "[{}] {} [{}] {}\n",
                    entry.timestamp.to_rfc3339(),
                    entry.level,
                    entry.target,
                    entry.message
                ));
            }
            text
        }
    };

    std::fs::write(&filepath, content)
        .map_err(|e| AdminError::ConfigError(format!("Failed to write log export: {}", e)))?;

    tracing::info!(
        "Logs exported to file: {} ({} entries)",
        filepath.display(),
        log_entries.entries.len()
    );
    Ok(filepath.to_string_lossy().to_string())
}

/// Get statistics about log entries
pub async fn get_log_stats(log_buffer: &SharedLogBuffer) -> Result<LogStats, AdminError> {
    let all_entries = log_buffer.get_all().await;
    let mut entries_by_level = HashMap::new();
    let mut entries_by_module = HashMap::new();

    for entry in &all_entries {
        *entries_by_level.entry(entry.level.clone()).or_insert(0) += 1;
        *entries_by_module.entry(entry.target.clone()).or_insert(0) += 1;
    }

    Ok(LogStats {
        total_entries: all_entries.len() as u64,
        entries_by_level,
        entries_by_module,
        oldest_entry: all_entries.first().map(|e| e.timestamp),
        newest_entry: all_entries.last().map(|e| e.timestamp),
    })
}
