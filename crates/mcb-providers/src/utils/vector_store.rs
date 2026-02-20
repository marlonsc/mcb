//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Vector Store Provider Utilities
//!
//! Shared utilities for vector store provider implementations (DRY principle).
//! Contains common HTTP error handling and response parsing patterns.

use std::collections::HashMap;
use std::time::Duration;

use mcb_domain::error::Error;
use mcb_domain::value_objects::{FileInfo, SearchResult};
use serde_json::Value;

use super::http::{RequestErrorKind, handle_request_error_with_kind};
use crate::constants::{
    VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LANGUAGE, VECTOR_FIELD_LINE_NUMBER,
    VECTOR_FIELD_START_LINE,
};

/// Handle HTTP request errors for vector store operations
///
/// Converts reqwest errors into domain errors with proper timeout detection.
#[must_use]
pub fn handle_vector_request_error(
    error: &reqwest::Error,
    timeout: Duration,
    provider: &str,
    operation: &str,
) -> Error {
    handle_request_error_with_kind(
        error,
        timeout,
        provider,
        operation,
        RequestErrorKind::VectorDb,
    )
}

/// Build a `SearchResult` from a JSON metadata/payload object.
///
/// Extracts `file_path`, `start_line`, `content`, and `language` fields using
/// the standard `VECTOR_FIELD_*` constants. Falls back to `line_number` when
/// `start_line` is absent.
///
/// Shared across Pinecone, Qdrant, and `EdgeVec` providers to avoid repeating
/// the same metadata field extraction logic.
#[must_use]
pub fn search_result_from_json_metadata(id: String, metadata: &Value, score: f64) -> SearchResult {
    SearchResult {
        id,
        file_path: metadata
            .get(VECTOR_FIELD_FILE_PATH)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        start_line: metadata
            .get(VECTOR_FIELD_START_LINE)
            .and_then(Value::as_u64)
            .or_else(|| {
                metadata
                    .get(VECTOR_FIELD_LINE_NUMBER)
                    .and_then(Value::as_u64)
            })
            .unwrap_or(0) as u32,
        content: metadata
            .get(VECTOR_FIELD_CONTENT)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        score,
        language: metadata
            .get(VECTOR_FIELD_LANGUAGE)
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned(),
    }
}

/// Build a list of `FileInfo` from search results by grouping on `file_path`.
///
/// This logic is shared across vector store providers (Pinecone, Qdrant, etc.)
/// whose `list_file_paths` implementation follows the same pattern:
/// call `list_vectors`, then aggregate results by file.
#[must_use]
pub fn build_file_info_from_results(results: Vec<SearchResult>) -> Vec<FileInfo> {
    let mut file_map: HashMap<String, (u32, String)> = HashMap::new();
    for result in results {
        let entry = file_map
            .entry(result.file_path)
            .or_insert_with(|| (0, result.language));
        entry.0 += 1;
    }

    file_map
        .into_iter()
        .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
        .collect()
}
