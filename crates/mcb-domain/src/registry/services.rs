use std::any::Any;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::ports::{
    AgentSessionServiceInterface, ContextServiceInterface, IndexingServiceInterface,
    MemoryServiceInterface, SearchServiceInterface, ValidationServiceInterface,
};

pub const CONTEXT_SERVICE_NAME: &str = "context";
pub const INDEXING_SERVICE_NAME: &str = "indexing";
pub const SEARCH_SERVICE_NAME: &str = "search";
pub const MEMORY_SERVICE_NAME: &str = "memory";
pub const AGENT_SESSION_SERVICE_NAME: &str = "agent_session";
pub const VALIDATION_SERVICE_NAME: &str = "validation";

#[derive(Clone, Copy)]
pub enum ServiceBuilder {
    Context(fn(&dyn Any) -> Result<Arc<dyn ContextServiceInterface>>),
    Indexing(fn(&dyn Any) -> Result<Arc<dyn IndexingServiceInterface>>),
    Search(fn(&dyn Any) -> Result<Arc<dyn SearchServiceInterface>>),
    Memory(fn(&dyn Any) -> Result<Arc<dyn MemoryServiceInterface>>),
    AgentSession(fn(&dyn Any) -> Result<Arc<dyn AgentSessionServiceInterface>>),
    Validation(fn(&dyn Any) -> Result<Arc<dyn ValidationServiceInterface>>),
}

pub struct ServiceRegistryEntry {
    pub name: &'static str,
    pub build: ServiceBuilder,
}

#[linkme::distributed_slice]
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
