//! DI resolution context — lives in mcb-domain so all layers can reference it.
//!
//! Fields for DB connection and app config are opaque (`Arc<dyn Any>`) because
//! mcb-domain must not depend on infrastructure crates. The concrete types
//! (e.g., `sea_orm::DatabaseConnection`, `AppConfig`) are downcast inside the
//! infrastructure service builders that actually need them.

use std::any::Any;
use std::sync::Arc;

use crate::ports::{EmbeddingProvider, EventBusProvider, VectorStoreProvider};

/// Context passed to service factory functions during DI resolution.
///
/// This structure implements the **Opaque DI** pattern, allowing `mcb-domain` to
/// define the required dependencies for service instantiation without
/// depending on the concrete types defined in `mcb-infrastructure` or `mcb-server`.
///
/// Each layer can create and pass this context through the registry, while
/// specialized service builders "downcast" the internal `Arc<dyn Any>` types
/// to the concrete implementations they require (such as `DatabaseConnection`).
pub struct ServiceResolutionContext {
    /// Active database connection (typically `sea_orm::DatabaseConnection`).
    /// Infrastructure service builders downcast to the concrete type.
    pub db: Arc<dyn Any + Send + Sync>,
    /// Shared application configuration (typically `AppConfig`).
    /// Infrastructure service builders downcast to the concrete type.
    pub config: Arc<dyn Any + Send + Sync>,
    /// Event bus for cross-service communication.
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Shared embedding provider resolved once at startup.
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Shared vector store provider resolved once at startup.
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
}
