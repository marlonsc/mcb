//! Pinecone provider factory and auto-registration.

use std::sync::Arc;

use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;

use super::PineconeVectorStoreProvider;

/// Factory function for creating Pinecone vector store provider instances.
///
/// # Errors
///
/// Returns `Err` if required configuration (API key, host) is missing.
pub fn pinecone_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| "Pinecone requires api_key".to_owned())?;
    let host = config
        .uri
        .clone()
        .ok_or_else(|| "Pinecone requires uri (index host URL)".to_owned())?;
    let http_client = create_default_client()?;

    Ok(Arc::new(PineconeVectorStoreProvider::new(
        &api_key,
        &host,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
        config.dimensions,
    )))
}

mcb_domain::register_vector_store_provider!(
    "pinecone",
    "Pinecone cloud vector database (managed, serverless)",
    pinecone_factory
);
