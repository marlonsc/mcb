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
use mcb_domain::registry::services::{ServiceBuilder, resolve_context_service};

use super::{IndexingServiceDeps, IndexingServiceImpl, IndexingServiceWithHashDeps};

use mcb_utils::constants::{
    DEFAULT_DATABASE_PROVIDER, DEFAULT_INDEXING_OP_PROVIDER, DEFAULT_LANGUAGE_PROVIDER,
    DEFAULT_NAMESPACE,
};

/// Build the `IndexingService` from the application registry.
///
/// # Errors
///
/// Returns an error if any required dependency cannot be resolved from the registry.
fn build_indexing_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn IndexingServiceInterface>> {
    let ctx = context
        .downcast_ref::<mcb_domain::registry::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Indexing registry builder requires ServiceResolutionContext",
            )
        })?;

    let app_config = ctx
        .config
        .downcast_ref::<crate::config::app::AppConfig>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Indexing service requires AppConfig in resolution context",
            )
        })?;
    let db = Arc::clone(&ctx.db);

    let context_service = resolve_context_service(context)?;
    let language_chunker =
        resolve_language_provider(&LanguageProviderConfig::new(DEFAULT_LANGUAGE_PROVIDER))?;

    // Use "seaorm" — the actual registry provider — not the user-facing config name.
    let repositories =
        resolve_database_repositories(DEFAULT_DATABASE_PROVIDER, db, DEFAULT_NAMESPACE.to_owned())?;

    let indexing_ops: Arc<dyn mcb_domain::ports::IndexingOperationsInterface> =
        resolve_indexing_operations_provider(&IndexingOperationsProviderConfig::new(
            DEFAULT_INDEXING_OP_PROVIDER,
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

mcb_domain::register_service!(
    mcb_utils::constants::SERVICE_NAME_INDEXING,
    ServiceBuilder::Indexing(build_indexing_service_from_registry),
);
