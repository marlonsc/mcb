//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Infrastructure factory helpers.
//!
//! Config-driven factories for contexts that don't need full `AppContext`.
//! Use these instead of importing concrete types from `mcb-providers`.

use std::sync::Arc;

use mcb_domain::ports::{EventBusProvider, VectorStoreBrowser, VectorStoreProvider};

/// Create the default in-process event bus for standalone contexts.
///
/// Uses `TokioEventBusProvider` — the same implementation resolved by the DI
/// bootstrap. Exported so that `mcb-server` can build a minimal `AdminState`
/// without a full `AppContext` while avoiding Null Object patterns.
#[must_use]
pub fn default_event_bus() -> Arc<dyn EventBusProvider> {
    Arc::new(mcb_providers::events::TokioEventBusProvider::new())
}

/// Create an in-memory vector store for browse/test flows.
///
/// Wraps `EdgeVecVectorStoreProvider` so consumer crates never import
/// `mcb_providers` directly. The returned trait object implements
/// `VectorStoreBrowser` (and `VectorStoreProvider` + `VectorStoreAdmin`).
///
/// # Errors
///
/// Returns an error if the in-memory vector store cannot be initialized.
pub fn create_test_browse_vector_store(
    dimensions: usize,
) -> mcb_domain::error::Result<Arc<dyn VectorStoreBrowser>> {
    use mcb_providers::vector_store::{EdgeVecConfig, EdgeVecVectorStoreProvider};

    let config = EdgeVecConfig {
        dimensions,
        ..Default::default()
    };
    let provider = EdgeVecVectorStoreProvider::new(&config)?;
    Ok(Arc::new(provider))
}

/// Create an in-memory vector store for E2E browse tests.
///
/// Returns both a [`VectorStoreProvider`] (for populating test data) and a
/// [`VectorStoreBrowser`] (for `BrowseState`) backed by the same underlying
/// store instance.
///
/// # Errors
///
/// Returns an error if the in-memory vector store cannot be initialized.
pub fn create_test_vector_store_for_e2e(
    dimensions: usize,
) -> mcb_domain::error::Result<(Arc<dyn VectorStoreProvider>, Arc<dyn VectorStoreBrowser>)> {
    use mcb_providers::vector_store::{EdgeVecConfig, EdgeVecVectorStoreProvider};

    let config = EdgeVecConfig {
        dimensions,
        ..Default::default()
    };
    let provider = Arc::new(EdgeVecVectorStoreProvider::new(&config)?);
    let browser: Arc<dyn VectorStoreBrowser> =
        std::sync::Arc::<EdgeVecVectorStoreProvider>::clone(&provider);
    let store: Arc<dyn VectorStoreProvider> = provider;
    Ok((store, browser))
}
