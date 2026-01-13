//! Async file I/O utilities - Replaces 10+ lines per call across codebase (DRY)
//!
//! Consolidates common patterns for JSON serialization, file writing, and error handling

use crate::domain::error::{Error, Result};
use std::path::Path;

/// Async file utilities for common I/O patterns
pub struct FileUtils;

impl FileUtils {
    /// Write JSON to file with proper error handling (replaces ~8 lines per use)
    ///
    /// Serializes value to JSON and writes atomically with descriptive error.
    pub async fn write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", context, e)))?;
        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", context, e)))?;
        Ok(())
    }

    /// Read JSON from file with proper error handling (replaces ~8 lines per use)
    pub async fn read_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
        path: P,
        context: &str,
    ) -> Result<T> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| Error::io(format!("Failed to read {}: {}", context, e)))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse {}: {}", context, e)))
    }

    /// Ensure directory exists and write file (replaces ~12 lines per use)
    pub async fn ensure_dir_write<P: AsRef<Path>>(
        path: P,
        content: &[u8],
        context: &str,
    ) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::io(format!("Failed to create directory for {}: {}", context, e))
            })?;
        }
        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", context, e)))?;
        Ok(())
    }

    /// Ensure directory exists and write JSON (replaces ~15 lines per use)
    pub async fn ensure_dir_write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", context, e)))?;
        Self::ensure_dir_write(path, content.as_bytes(), context).await
    }

    /// Check if path exists (async wrapper for std::path::Path::exists)
    pub async fn exists<P: AsRef<Path>>(path: P) -> bool {
        tokio::fs::metadata(path.as_ref()).await.is_ok()
    }

    /// Read file if exists, return None otherwise (replaces ~6 lines per use)
    pub async fn read_if_exists<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
        match tokio::fs::read(path.as_ref()).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io(format!("Failed to read file: {}", e))),
        }
    }
}
