//! File utility helpers for filesystem vector store
//!
//! Async file operations for reading and writing JSON data.

use mcb_domain::error::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

/// Check if a file or directory exists at the given path.
pub async fn exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

/// Read and deserialize JSON data from a file.
pub async fn read_json<T: DeserializeOwned>(path: &Path, description: &str) -> Result<T> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::io(format!("Failed to read {}: {}", description, e)))?;
    serde_json::from_str(&content)
        .map_err(|e| Error::internal(format!("Failed to parse {}: {}", description, e)))
}

/// Serialize and write data as JSON to a file.
pub async fn write_json<T: Serialize>(path: &Path, data: &T, description: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", description, e)))?;
    tokio::fs::write(path, content)
        .await
        .map_err(|e| Error::io(format!("Failed to write {}: {}", description, e)))
}

/// Write bytes to a file, creating parent directories if needed.
pub async fn ensure_dir_write(path: &Path, data: &[u8], description: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            Error::io(format!(
                "Failed to create directory for {}: {}",
                description, e
            ))
        })?;
    }
    tokio::fs::write(path, data)
        .await
        .map_err(|e| Error::io(format!("Failed to write {}: {}", description, e)))
}

/// Write JSON data to a file, creating parent directories if needed.
pub async fn ensure_dir_write_json<T: Serialize>(
    path: &Path,
    data: &T,
    description: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            Error::io(format!(
                "Failed to create directory for {}: {}",
                description, e
            ))
        })?;
    }
    write_json(path, data, description).await
}
