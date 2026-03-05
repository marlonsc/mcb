//! Milvus provider factory and auto-registration.

use super::MilvusVectorStoreProvider;

mcb_domain::register_vector_store_provider!(
    mcb_utils::constants::PROVIDER_SLUG_MILVUS,
    "Milvus distributed vector database",
    milvus_factory
);

fn milvus_factory(
    config: &mcb_domain::registry::vector_store::VectorStoreProviderConfig,
) -> std::result::Result<std::sync::Arc<dyn mcb_domain::ports::VectorStoreProvider>, String> {
    let uri = config.uri.clone().ok_or_else(|| {
        format!(
            "Milvus requires 'uri' configuration (e.g., http://localhost:{})",
            mcb_utils::constants::vector_store::MILVUS_DEFAULT_PORT
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
