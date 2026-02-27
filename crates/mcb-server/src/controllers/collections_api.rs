//! Collections API controller — returns vector store collection info as JSON.

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

/// Registers collections API routes.
#[must_use]
pub fn routes() -> Routes {
    Routes::new()
        .prefix("collections")
        .add("/", get(collections))
}
