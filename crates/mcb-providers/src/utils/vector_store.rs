//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Vector Store Provider Utilities
//!
//! Shared utilities for vector store provider implementations (DRY principle).
//! Contains common HTTP error handling and response parsing patterns.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use super::http::{
    RequestErrorKind, VectorDbRequestParams, handle_request_error_with_kind, send_vector_db_request,
};
use mcb_utils::constants::http::{CONTENT_TYPE_JSON, HTTP_HEADER_CONTENT_TYPE};
use mcb_utils::constants::vector_store::{
    VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LANGUAGE, VECTOR_FIELD_START_LINE,
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

/// Shared `list_file_paths` helper for vector store providers whose
/// implementation follows the standard pattern: call `list_vectors`, then
/// aggregate results via `build_file_info_from_results`.
///
/// Providers with custom logic (Milvus, `EdgeVec`, Encrypted) should implement
/// `list_file_paths` directly instead of delegating to this function.
///
/// # Errors
///
/// Returns an error if the underlying `list_vectors` call fails.
pub async fn standard_list_file_paths(
    provider: &impl mcb_domain::ports::VectorStoreProvider,
    collection: &mcb_domain::value_objects::CollectionId,
    limit: usize,
) -> Result<Vec<FileInfo>> {
    let results = provider.list_vectors(collection, limit).await?;
    Ok(build_file_info_from_results(results))
}

/// Normalize a base URL by trimming trailing slashes.
///
/// Shared by the Pinecone, Qdrant, and Weaviate constructors.
#[must_use]
pub fn normalize_base_url(url: &str) -> String {
    url.trim_end_matches('/').to_owned()
}

/// Normalize an optional API key by trimming surrounding whitespace.
///
/// Shared by the Qdrant and Weaviate constructors.
#[must_use]
pub fn normalize_api_key(key: Option<String>) -> Option<String> {
    key.map(|k| k.trim().to_owned())
}

/// Create an empty collections dimension map shared by HTTP vector store providers.
#[must_use]
pub fn new_collections_map() -> Arc<DashMap<String, usize>> {
    Arc::new(DashMap::new())
}

/// Build the standard equality filter on the file-path vector field.
///
/// Shared browse scaffolding: every provider filters chunks of one file the
/// same way; only the transport of that filter differs per API.
#[must_use]
pub fn file_path_eq_filter(file_path: &str) -> Value {
    serde_json::json!({ VECTOR_FIELD_FILE_PATH: { "$eq": file_path } })
}

/// Sort search results by ascending start line (standard browse ordering).
pub fn sort_by_start_line(results: &mut [SearchResult]) {
    results.sort_by_key(|r| r.start_line);
}

/// Shared HTTP plumbing for the REST vector store providers (Pinecone,
/// Qdrant, Weaviate): normalized base URL, optional API key, request timeout,
/// HTTP client, and the local collections dimension map.
///
/// Each provider embeds this core and supplies only its own authentication
/// header shape through [`HttpVectorStoreCore::request`].
pub(crate) struct HttpVectorStoreCore {
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) timeout: Duration,
    pub(crate) http_client: Client,
    pub(crate) collections: Arc<DashMap<String, usize>>,
}

impl HttpVectorStoreCore {
    /// Create the shared core with normalized URL and API key.
    #[must_use]
    pub(crate) fn new(
        base_url: &str,
        api_key: Option<String>,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        Self {
            base_url: normalize_base_url(base_url),
            api_key: normalize_api_key(api_key),
            timeout,
            http_client,
            collections: new_collections_map(),
        }
    }

    /// Build a full URL for an API path.
    #[must_use]
    pub(crate) fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Send an authenticated JSON request through the shared scaffolding.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the HTTP request fails or the response is
    /// not a valid JSON payload.
    pub(crate) async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&Value>,
        provider: &str,
        auth_headers: Vec<(&'static str, String)>,
    ) -> Result<Value> {
        send_provider_request(ProviderRequest {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider,
            operation: path,
            auth_headers,
            body,
        })
        .await
    }
}

/// Parameters for [`send_provider_request`].
pub(crate) struct ProviderRequest<'a> {
    pub client: &'a Client,
    pub method: reqwest::Method,
    pub url: String,
    pub timeout: Duration,
    pub provider: &'a str,
    pub operation: &'a str,
    /// Provider-specific authentication headers; `Content-Type` is appended here.
    pub auth_headers: Vec<(&'static str, String)>,
    pub body: Option<&'a Value>,
}

/// Send an authenticated JSON request to a vector store provider.
///
/// Appends the standard JSON `Content-Type` header to the provider-specific
/// authentication headers and delegates to [`send_vector_db_request`] for
/// transport and error mapping.
///
/// # Errors
///
/// Returns a domain error when the HTTP request fails or the response is not
/// a valid JSON payload.
pub(crate) async fn send_provider_request(params: ProviderRequest<'_>) -> Result<Value> {
    let ProviderRequest {
        client,
        method,
        url,
        timeout,
        provider,
        operation,
        mut auth_headers,
        body,
    } = params;
    auth_headers.push((HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned()));

    send_vector_db_request(VectorDbRequestParams {
        client,
        method,
        url,
        timeout,
        provider,
        operation,
        headers: &auth_headers,
        body,
    })
    .await
}
