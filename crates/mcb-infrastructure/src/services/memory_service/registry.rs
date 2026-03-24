//! Service registry and linkme registration for `MemoryService`.
//!
//! Handles dependency resolution and service builder registration.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::registry::services::ServiceBuilder;

use super::MemoryServiceImpl;

use mcb_utils::constants::{DEFAULT_DATABASE_PROVIDER, DEFAULT_NAMESPACE};

/// Build a `MemoryService` from the service resolution context.
fn build_memory_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn MemoryServiceInterface>> {
    let ctx = context
        .downcast_ref::<mcb_domain::registry::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Memory service builder requires ServiceResolutionContext",
            )
        })?;

    let embedding = Arc::clone(&ctx.embedding_provider);
    let vector_store = Arc::clone(&ctx.vector_store_provider);

    // Resolve memory repository from database providers
    let repos = mcb_domain::registry::database::resolve_database_repositories(
        DEFAULT_DATABASE_PROVIDER,
        Arc::clone(&ctx.db),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    Ok(Arc::new(MemoryServiceImpl::new(
        DEFAULT_NAMESPACE.to_owned(),
        repos.memory,
        embedding,
        vector_store,
    )))
}

mcb_domain::register_service!(
    mcb_utils::constants::SERVICE_NAME_MEMORY,
    ServiceBuilder::Memory(build_memory_service_from_registry),
);
