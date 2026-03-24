//! Qdrant provider factory and auto-registration.

use super::QdrantVectorStoreProvider;
use mcb_utils::constants::vector_store::QDRANT_DEFAULT_PORT;

mcb_domain::register_vector_store_provider!(
    mcb_utils::constants::PROVIDER_SLUG_QDRANT,
    "Qdrant vector search engine (open-source, cloud and self-hosted)",
    qdrant_factory
);

fn qdrant_factory(
    config: &mcb_domain::registry::vector_store::VectorStoreProviderConfig,
) -> std::result::Result<std::sync::Arc<dyn mcb_domain::ports::VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let base_url = config
        .uri
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{QDRANT_DEFAULT_PORT}"));
    let api_key = config.api_key.clone();
    let http_client = create_default_client()?;

    Ok(std::sync::Arc::new(QdrantVectorStoreProvider::new(
        &base_url,
        api_key,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}
