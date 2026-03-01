//! Service registry and linkme registration for `MemoryService`.
//!
//! Handles dependency resolution and service builder registration.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::registry::services::{
    MEMORY_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
};

use super::MemoryServiceImpl;

/// Registry provider name for `SeaORM` database repositories.
const DATABASE_PROVIDER: &str = "seaorm";

/// Default namespace for database repositories.
const DEFAULT_NAMESPACE: &str = "default";

/// Build a `MemoryService` from the service resolution context.
fn build_memory_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn MemoryServiceInterface>> {
    let ctx = context
        .downcast_ref::<crate::resolution_context::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Memory service builder requires ServiceResolutionContext",
            )
        })?;

    let embedding = Arc::clone(&ctx.embedding_provider);
    let vector_store = Arc::clone(&ctx.vector_store_provider);

    // Resolve memory repository from database providers
    let repos = mcb_domain::registry::database::resolve_database_repositories(
        DATABASE_PROVIDER,
        Box::new(ctx.db.clone()),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    Ok(Arc::new(MemoryServiceImpl::new(
        DEFAULT_NAMESPACE.to_owned(),
        repos.memory,
        embedding,
        vector_store,
    )))
}

/// Linkme distributed slice entry for `MemoryService` registration.
#[linkme::distributed_slice(SERVICES_REGISTRY)]
static MEMORY_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: MEMORY_SERVICE_NAME,
    build: ServiceBuilder::Memory(build_memory_service_from_registry),
};
