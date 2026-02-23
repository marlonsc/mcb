//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Infrastructure factory helpers.
//!
//! Config-driven factories for contexts that don't need full `AppContext`.
//! Use these instead of importing concrete types from `mcb-providers`.

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{EventBusProvider, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::registry::event_bus::{EventBusProviderConfig, resolve_event_bus_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};

struct NullEventBusProvider;

struct VectorStoreBrowserAdapter {
    inner: Arc<dyn VectorStoreProvider>,
}

#[async_trait]
impl VectorStoreBrowser for VectorStoreBrowserAdapter {
    async fn list_collections(
        &self,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::value_objects::CollectionInfo>> {
        self.inner.list_collections().await
    }

    async fn list_file_paths(
        &self,
        collection: &mcb_domain::value_objects::CollectionId,
        limit: usize,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::value_objects::FileInfo>> {
        self.inner.list_file_paths(collection, limit).await
    }

    async fn get_chunks_by_file(
        &self,
        collection: &mcb_domain::value_objects::CollectionId,
        file_path: &str,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::value_objects::SearchResult>> {
        self.inner.get_chunks_by_file(collection, file_path).await
    }
}

#[async_trait]
impl EventBusProvider for NullEventBusProvider {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<mcb_domain::ports::DomainEventStream> {
        Ok(Box::pin(stream::empty::<DomainEvent>()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("null-event-bus".to_owned())
    }
}

/// Create the default in-process event bus for standalone contexts.
///
/// Uses `TokioEventBusProvider` — the same implementation resolved by the DI
/// bootstrap. Exported so that `mcb-server` can build a minimal `AdminState`
/// without a full `AppContext` while avoiding Null Object patterns.
#[must_use]
pub fn default_event_bus() -> Arc<dyn EventBusProvider> {
    resolve_event_bus_provider(&EventBusProviderConfig::new("tokio"))
        .unwrap_or_else(|_| Arc::new(NullEventBusProvider))
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
    let store = resolve_vector_store_provider(
        &VectorStoreProviderConfig::new("edgevec")
            .with_dimensions(dimensions)
            .with_collection("test-collection"),
    )?;
    Ok(Arc::new(VectorStoreBrowserAdapter { inner: store }))
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
    let config = VectorStoreProviderConfig::new("edgevec")
        .with_dimensions(dimensions)
        .with_collection("test-collection");
    let store = resolve_vector_store_provider(&config)?;
    let browser: Arc<dyn VectorStoreBrowser> = Arc::new(VectorStoreBrowserAdapter {
        inner: Arc::clone(&store),
    });
    Ok((store, browser))
}
