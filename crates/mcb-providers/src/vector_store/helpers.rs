//! Vector Store Provider Helpers
//!
//! Shared utilities for vector store provider implementations (DRY principle).
//! Contains common HTTP error handling and response parsing patterns.

use std::time::Duration;

use mcb_domain::error::Error;

use crate::utils::http::{RequestErrorKind, handle_request_error_with_kind};

/// Handle HTTP request errors for vector store operations
///
/// Converts reqwest errors into domain errors with proper timeout detection.
///
/// # Arguments
/// * `error` - The reqwest error to handle
/// * `timeout` - The timeout duration for error messages
/// * `provider` - The provider name for error context
/// * `operation` - The operation being performed (e.g., "search", "upsert")
///
/// # Returns
/// Domain Error with appropriate message
pub fn handle_vector_request_error(
    error: reqwest::Error,
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

/// Build a list of `FileInfo` from search results by grouping on `file_path`.
///
/// This logic is shared across vector store providers (Pinecone, Qdrant, etc.)
/// whose `list_file_paths` implementation follows the same pattern:
/// call `list_vectors`, then aggregate results by file.
pub fn build_file_info_from_results(
    results: Vec<mcb_domain::value_objects::SearchResult>,
) -> Vec<mcb_domain::value_objects::FileInfo> {
    use std::collections::HashMap;

    let mut file_map: HashMap<String, (u32, String)> = HashMap::new();
    for result in results {
        let entry = file_map
            .entry(result.file_path)
            .or_insert_with(|| (0, result.language));
        entry.0 += 1;
    }

    file_map
        .into_iter()
        .map(|(path, (chunk_count, language))| {
            mcb_domain::value_objects::FileInfo::new(path, chunk_count, language, None)
        })
        .collect()
}
