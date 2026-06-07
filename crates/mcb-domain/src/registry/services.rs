use std::any::Any;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::ports::{
    AgentSessionServiceInterface, ContextServiceInterface, HighlightServiceInterface,
    IndexingServiceInterface, MemoryServiceInterface, SearchServiceInterface,
    ValidationServiceInterface,
};

/// Registry name for the context service.
pub const CONTEXT_SERVICE_NAME: &str = "context";
/// Registry name for the indexing service.
pub const INDEXING_SERVICE_NAME: &str = "indexing";
/// Registry name for the search service.
pub const SEARCH_SERVICE_NAME: &str = "search";
/// Registry name for the memory service.
pub const MEMORY_SERVICE_NAME: &str = "memory";
/// Registry name for the agent session service.
pub const AGENT_SESSION_SERVICE_NAME: &str = "agent_session";
/// Registry name for the validation service.
pub const VALIDATION_SERVICE_NAME: &str = "validation";
/// Registry name for the highlight service.
pub const HIGHLIGHT_SERVICE_NAME: &str = "highlight";

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
    /// Registry name (must match one of the `*_SERVICE_NAME` constants).
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

macro_rules! resolve_service {
    ($fn_name:ident, $const:ident, $variant:ident, $trait_obj:ty) => {
        /// Resolve a service by name from the registry.
        ///
        /// # Errors
        ///
        /// Returns an error if the service provider is not registered or has the wrong variant.
        pub fn $fn_name(context: &dyn std::any::Any) -> Result<std::sync::Arc<$trait_obj>> {
            match find_builder($const)? {
                ServiceBuilder::$variant(build) => build(context),
                _ => Err(Error::internal(format!(
                    "Service provider '{}' is not a {} builder",
                    $const,
                    stringify!($variant)
                ))),
            }
        }
    };
}

resolve_service!(
    resolve_context_service,
    CONTEXT_SERVICE_NAME,
    Context,
    dyn ContextServiceInterface
);
resolve_service!(
    resolve_indexing_service,
    INDEXING_SERVICE_NAME,
    Indexing,
    dyn IndexingServiceInterface
);
resolve_service!(
    resolve_search_service,
    SEARCH_SERVICE_NAME,
    Search,
    dyn SearchServiceInterface
);
resolve_service!(
    resolve_memory_service,
    MEMORY_SERVICE_NAME,
    Memory,
    dyn MemoryServiceInterface
);
resolve_service!(
    resolve_agent_session_service,
    AGENT_SESSION_SERVICE_NAME,
    AgentSession,
    dyn AgentSessionServiceInterface
);
resolve_service!(
    resolve_validation_service,
    VALIDATION_SERVICE_NAME,
    Validation,
    dyn ValidationServiceInterface
);
resolve_service!(
    resolve_highlight_service,
    HIGHLIGHT_SERVICE_NAME,
    Highlight,
    dyn HighlightServiceInterface
);
