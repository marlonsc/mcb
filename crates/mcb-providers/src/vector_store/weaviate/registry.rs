//! Weaviate provider factory and auto-registration.

use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;

use super::WeaviateVectorStoreProvider;

/// Factory function for creating Weaviate vector store provider instances.
///
/// # Errors
///
/// Returns `Err` if required configuration (uri) is missing.
pub fn weaviate_factory(
    config: &VectorStoreProviderConfig,
) -> Result<Arc<dyn VectorStoreProvider>> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let uri = config
        .uri
        .clone()
        .ok_or_else(|| Error::configuration("Weaviate requires uri (http://host:8080)"))?;
    let http_client = create_default_client()?;

    Ok(Arc::new(WeaviateVectorStoreProvider::new(
        &uri,
        config.api_key.clone(),
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

mcb_domain::register_vector_store_provider!(
    "weaviate",
    "Weaviate vector database (REST + GraphQL, native multi-tenancy)",
    weaviate_factory
);
