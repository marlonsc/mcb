//! `EdgeVec` provider factory and auto-registration.

use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;
use mcb_domain::value_objects::CollectionId;

use super::EdgeVecVectorStoreProvider;
use super::config::EdgeVecConfig;

/// Factory function for creating `EdgeVec` vector store provider instances.
fn edgevec_factory(config: &VectorStoreProviderConfig) -> Result<Arc<dyn VectorStoreProvider>> {
    let dimensions = config
        .dimensions
        .unwrap_or(mcb_utils::constants::embedding::EMBEDDING_DIMENSION_NULL);
    let collection_name = config.collection.clone().ok_or_else(|| {
        Error::configuration("EdgeVec provider requires a collection name in vector_store config")
    })?;
    let edgevec_config = EdgeVecConfig {
        dimensions,
        ..Default::default()
    };
    let provider = EdgeVecVectorStoreProvider::with_collection(
        &edgevec_config,
        CollectionId::from_name(&collection_name),
    )
    .map_err(|e| Error::vector_db(format!("Failed to create EdgeVec provider: {e}")))?;
    Ok(Arc::new(provider))
}

mcb_domain::register_vector_store_provider!(
    mcb_utils::constants::PROVIDER_SLUG_EDGEVEC,
    "EdgeVec in-memory HNSW vector store (high-performance)",
    edgevec_factory
);
