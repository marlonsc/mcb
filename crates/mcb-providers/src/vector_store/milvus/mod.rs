//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Milvus vector store provider implementation
//!
//! High-performance cloud vector database using Milvus.
//! Supports production-scale vector storage with automatic indexing and distributed search.

use mcb_domain::error::Result;
use mcb_domain::value_objects::CollectionId;
use milvus::client::Client;

use crate::constants::MILVUS_DEFAULT_TIMEOUT_SECS;

/// Milvus admin operations (create, drop, health).
pub mod admin;
/// Milvus collection browsing operations.
pub mod browser;
mod helpers;
mod list;
mod provider;
/// Schema utilities for Milvus collections.
pub mod schema;
mod search;

/// Milvus vector store provider implementation
pub struct MilvusVectorStoreProvider {
    client: Client,
}

/// Default output fields for Milvus queries.
pub const DEFAULT_OUTPUT_FIELDS: &[&str] = &[
    crate::constants::VECTOR_FIELD_ID,
    crate::constants::VECTOR_FIELD_FILE_PATH,
    crate::constants::VECTOR_FIELD_START_LINE,
    crate::constants::VECTOR_FIELD_CONTENT,
];

/// Convert a `CollectionId` to a valid Milvus collection name.
///
/// Milvus requires collection names matching `^[a-zA-Z_][a-zA-Z0-9_]*$` (max 255 chars).
/// UUIDs (e.g. `2f106fbd-e15a-5304-8adf-75e1ab8ba3ee`) are converted by:
///   1. Stripping hyphens -> `2f106fbde15a53048adf75e1ab8ba3ee`
///   2. Prefixing with `mcb_` -> `mcb_2f106fbde15a53048adf75e1ab8ba3ee`
#[must_use]
pub fn to_milvus_name(collection: &CollectionId) -> String {
    let raw = collection.to_string();
    let sanitized = raw.replace('-', "");
    format!("mcb_{sanitized}")
}

pub(super) fn is_collection_not_found(msg: &str) -> bool {
    msg.contains(crate::constants::MILVUS_ERROR_COLLECTION_NOT_EXISTS)
        || msg.contains("collection not found")
        || msg.contains("not exist")
}

impl MilvusVectorStoreProvider {
    /// Helper method to convert Milvus errors to domain errors
    pub(super) fn map_milvus_error<T, E: std::fmt::Display>(
        result: std::result::Result<T, E>,
        operation: &str,
    ) -> Result<T> {
        result
            .map_err(|e| mcb_domain::error::Error::vector_db(format!("Failed to {operation}: {e}")))
    }

    /// Create a new Milvus vector store provider
    ///
    /// # Arguments
    /// * `address` - Milvus server address (e.g., "<http://localhost:19530>")
    /// * `token` - Optional authentication token
    /// * `timeout_secs` - Connection timeout in seconds (default: 10)
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Milvus server fails.
    pub async fn new(
        address: String,
        _token: Option<String>,
        timeout_secs: Option<u64>,
    ) -> Result<Self> {
        // Ensure the address has a scheme (required by tonic transport)
        let endpoint = if address.starts_with("http://") || address.starts_with("https://") {
            address
        } else {
            format!("http://{address}")
        };

        let timeout = timeout_secs.unwrap_or(MILVUS_DEFAULT_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout);

        let client = tokio::time::timeout(timeout_duration, Client::new(endpoint.clone()))
            .await
            .map_err(|_| {
                mcb_domain::error::Error::vector_db(format!(
                    "Milvus connection timed out after {timeout} seconds"
                ))
            })?
            .map_err(|e| {
                mcb_domain::error::Error::vector_db(format!("Failed to connect to Milvus: {e}"))
            })?;

        Ok(Self { client })
    }

    pub(super) fn default_output_fields() -> Vec<String> {
        DEFAULT_OUTPUT_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect()
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

crate::register_vector_store_provider!(
    milvus_factory,
    config,
    MILVUS_PROVIDER,
    "milvus",
    "Milvus distributed vector database",
    {
        let uri = config.uri.clone().ok_or_else(|| {
            format!(
                "Milvus requires 'uri' configuration (e.g., http://localhost:{})",
                crate::constants::MILVUS_DEFAULT_PORT
            )
        })?;
        let token = config.api_key.clone();

        // Create Milvus client synchronously using block_on
        let provider = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { MilvusVectorStoreProvider::new(uri, token, None).await })
        })
        .map_err(|e| format!("Failed to create Milvus provider: {e}"))?;

        Ok(std::sync::Arc::new(provider))
    }
);
