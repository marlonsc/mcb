//! Collections API controller — returns vector store collection info as JSON.

use mcb_domain::value_objects::CollectionId;

use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;

/// Returns a list of all vector store collections.
///
/// Calls `VectorStoreBrowser::list_collections()` on the shared
/// `VectorStoreProvider` from `McbState`.
///
/// # Errors
///
/// Returns an empty list if the provider is unavailable (graceful degradation).
pub async fn collections(Extension(state): Extension<McbState>) -> Result<Response> {
    let collections = state
        .vector_store
        .list_collections()
        .await
        .unwrap_or_default();

    format::json(collections)
}

/// Returns all code chunks from all collections — used by the Browse UI.
///
/// Iterates every collection via `list_collections()`, then calls
/// `list_vectors(id, 50)` to retrieve up to 50 chunks per collection.
///
/// # Errors
///
/// Returns an empty list if the provider is unavailable (graceful degradation).
pub async fn chunks(Extension(state): Extension<McbState>) -> Result<Response> {
    let collections = state
        .vector_store
        .list_collections()
        .await
        .unwrap_or_default();

    let mut all_chunks = Vec::new();
    for collection in &collections {
        let id = CollectionId::from_string(&collection.name);
        let vecs = state
            .vector_store
            .list_vectors(&id, 50)
            .await
            .unwrap_or_default();
        all_chunks.extend(vecs);
    }

    format::json(all_chunks)
}

/// Registers collections API routes.
#[must_use]
pub fn routes() -> Routes {
    Routes::new()
        .prefix("collections")
        .add("/", get(collections))
}
