use std::any::Any;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::ports::{
    AgentSessionServiceInterface, ContextServiceInterface, HighlightServiceInterface,
    IndexingServiceInterface, MemoryServiceInterface, SearchServiceInterface,
    ValidationServiceInterface,
};

/// Typed factory enum for building domain services from a resolution context.
#[derive(Clone, Copy)]
pub enum ServiceBuilder {
    /// Build a context service.
    Context(fn(&dyn Any) -> Result<Arc<dyn ContextServiceInterface>>),
    /// Build an indexing service.
    Indexing(fn(&dyn Any) -> Result<Arc<dyn IndexingServiceInterface>>),
    /// Build a search service.
    Search(fn(&dyn Any) -> Result<Arc<dyn SearchServiceInterface>>),
    /// Build a memory service.
    Memory(fn(&dyn Any) -> Result<Arc<dyn MemoryServiceInterface>>),
    /// Build an agent session service.
    AgentSession(fn(&dyn Any) -> Result<Arc<dyn AgentSessionServiceInterface>>),
    /// Build a validation service.
    Validation(fn(&dyn Any) -> Result<Arc<dyn ValidationServiceInterface>>),
    /// Build a highlight service.
    Highlight(fn(&dyn Any) -> Result<Arc<dyn HighlightServiceInterface>>),
}

/// Entry in the service registry pairing a name with its builder.
pub struct ServiceRegistryEntry {
    /// Registry name (must match one of the `SERVICE_NAME_*` constants in `mcb_utils`).
    pub name: &'static str,
    /// Factory for this service.
    pub build: ServiceBuilder,
}

#[linkme::distributed_slice]
/// Distributed slice holding all service registry entries.
pub static SERVICES_REGISTRY: [ServiceRegistryEntry] = [..];

fn find_builder(name: &str) -> Result<ServiceBuilder> {
    SERVICES_REGISTRY
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.build)
        .ok_or_else(|| Error::internal(format!("No service provider found for '{name}'")))
}

// `resolve_service!` macro is defined in `crate::macros::services` and available via `#[macro_use]`.

resolve_service!(
    resolve_context_service,
    mcb_utils::constants::SERVICE_NAME_CONTEXT,
    Context,
    dyn ContextServiceInterface
);
resolve_service!(
    resolve_indexing_service,
    mcb_utils::constants::SERVICE_NAME_INDEXING,
    Indexing,
    dyn IndexingServiceInterface
);
resolve_service!(
    resolve_search_service,
    mcb_utils::constants::SERVICE_NAME_SEARCH,
    Search,
    dyn SearchServiceInterface
);
resolve_service!(
    resolve_memory_service,
    mcb_utils::constants::SERVICE_NAME_MEMORY,
    Memory,
    dyn MemoryServiceInterface
);
resolve_service!(
    resolve_agent_session_service,
    mcb_utils::constants::SERVICE_NAME_AGENT_SESSION,
    AgentSession,
    dyn AgentSessionServiceInterface
);
resolve_service!(
    resolve_validation_service,
    mcb_utils::constants::SERVICE_NAME_VALIDATION,
    Validation,
    dyn ValidationServiceInterface
);
resolve_service!(
    resolve_highlight_service,
    mcb_utils::constants::SERVICE_NAME_HIGHLIGHT,
    Highlight,
    dyn HighlightServiceInterface
);
