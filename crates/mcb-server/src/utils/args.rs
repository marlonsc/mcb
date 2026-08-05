//! Common argument-resolution helpers for MCP tool handlers.
//!
//! Centralizes repeated default-value and workspace-path resolution patterns
//! so handlers stay thin and focused on domain orchestration.

use std::path::PathBuf;

use rmcp::ErrorData as McpError;

/// Resolve an optional `limit` argument to a `usize`, applying a default.
#[must_use]
pub fn resolve_limit(limit: Option<u32>, default: u32) -> usize {
    limit.unwrap_or(default) as usize
}

/// Resolve a workspace path from an optional argument, falling back to the
/// current working directory, and validate that it exists and is a directory.
///
/// # Errors
/// Returns `McpError::invalid_params` when the path is missing/empty, the
/// working directory is unavailable, or the resolved path is not an existing
/// directory.
pub fn resolve_workspace_path(path: Option<&str>) -> Result<PathBuf, McpError> {
    let resolved = path
        .filter(|p| !p.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| {
            McpError::invalid_params("path is required (working directory unavailable)", None)
        })?;

    if !resolved.exists() {
        return Err(McpError::invalid_params(
            "Specified path does not exist",
            None,
        ));
    }
    if !resolved.is_dir() {
        return Err(McpError::invalid_params(
            "Specified path is not a directory",
            None,
        ));
    }

    Ok(resolved)
}
