//! Service registry and dependency resolution for `IndexingService`.
//!
//! This module handles the linkme-based service registration and factory function
//! for building the `IndexingService` from the application registry.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::IndexingServiceInterface;
use mcb_domain::registry::admin_operations::{
    IndexingOperationsProviderConfig, resolve_indexing_operations_provider,
};
use mcb_domain::registry::database::resolve_database_repositories;
use mcb_domain::registry::language::{LanguageProviderConfig, resolve_language_provider};
use mcb_domain::registry::services::{
    INDEXING_SERVICE_NAME, ServiceBuilder, ServiceRegistryEntry, resolve_context_service,
};

use super::{IndexingServiceDeps, IndexingServiceImpl, IndexingServiceWithHashDeps};

/// Registry provider name for `SeaORM` database repositories.
const DATABASE_PROVIDER: &str = "seaorm";

/// Default namespace for database repositories.
const DEFAULT_NAMESPACE: &str = "default";

/// Registry provider name for universal language chunking.
const LANGUAGE_PROVIDER: &str = "universal";

/// Build the `IndexingService` from the application registry.
///
/// # Errors
///
/// Returns an error if any required dependency cannot be resolved from the registry.
fn build_indexing_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn IndexingServiceInterface>> {
    let ctx = context
        .downcast_ref::<crate::resolution_context::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Indexing registry builder requires ServiceResolutionContext",
            )
        })?;

    let app_config = &ctx.config;
    let db = ctx.db.clone();

    let context_service = resolve_context_service(context)?;
    let language_chunker =
        resolve_language_provider(&LanguageProviderConfig::new(LANGUAGE_PROVIDER))?;

    // Use "seaorm" — the actual registry provider — not the user-facing config name.
    let repositories = resolve_database_repositories(
        DATABASE_PROVIDER,
        Box::new(db),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    let indexing_ops: Arc<dyn mcb_domain::ports::IndexingOperationsInterface> =
        resolve_indexing_operations_provider(&IndexingOperationsProviderConfig::new(
            DEFAULT_NAMESPACE,
        ))?;
    let event_bus = Arc::clone(&ctx.event_bus);

    Ok(Arc::new(
        IndexingServiceImpl::new_with_file_hash_repository(IndexingServiceWithHashDeps {
            service: IndexingServiceDeps {
                context_service,
                language_chunker,
                indexing_ops,
                event_bus,
                supported_extensions: app_config.mcp.indexing.supported_extensions.clone(),
            },
            file_hash_repository: repositories.file_hash,
        }),
    ))
}

/// Registry entry for the `IndexingService`.
///
/// This is registered via linkme's distributed slice mechanism, allowing
/// the service to be discovered and instantiated at runtime without explicit
/// registration code.
// linkme distributed_slice uses unsafe link-section attributes internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(mcb_domain::registry::services::SERVICES_REGISTRY)]
static INDEXING_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: INDEXING_SERVICE_NAME,
    build: ServiceBuilder::Indexing(build_indexing_service_from_registry),
};
