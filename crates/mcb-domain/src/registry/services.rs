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

pub fn resolve_context_service(context: &dyn Any) -> Result<Arc<dyn ContextServiceInterface>> {
    match find_builder(CONTEXT_SERVICE_NAME)? {
        ServiceBuilder::Context(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not a context builder",
            CONTEXT_SERVICE_NAME
        ))),
    }
}

pub fn resolve_indexing_service(context: &dyn Any) -> Result<Arc<dyn IndexingServiceInterface>> {
    match find_builder(INDEXING_SERVICE_NAME)? {
        ServiceBuilder::Indexing(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not an indexing builder",
            INDEXING_SERVICE_NAME
        ))),
    }
}

pub fn resolve_search_service(context: &dyn Any) -> Result<Arc<dyn SearchServiceInterface>> {
    match find_builder(SEARCH_SERVICE_NAME)? {
        ServiceBuilder::Search(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not a search builder",
            SEARCH_SERVICE_NAME
        ))),
    }
}

pub fn resolve_memory_service(context: &dyn Any) -> Result<Arc<dyn MemoryServiceInterface>> {
    match find_builder(MEMORY_SERVICE_NAME)? {
        ServiceBuilder::Memory(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not a memory builder",
            MEMORY_SERVICE_NAME
        ))),
    }
}

pub fn resolve_agent_session_service(
    context: &dyn Any,
) -> Result<Arc<dyn AgentSessionServiceInterface>> {
    match find_builder(AGENT_SESSION_SERVICE_NAME)? {
        ServiceBuilder::AgentSession(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not an agent session builder",
            AGENT_SESSION_SERVICE_NAME
        ))),
    }
}

pub fn resolve_validation_service(
    context: &dyn Any,
) -> Result<Arc<dyn ValidationServiceInterface>> {
    match find_builder(VALIDATION_SERVICE_NAME)? {
        ServiceBuilder::Validation(build) => build(context),
        _ => Err(Error::internal(format!(
            "Service provider '{}' is not a validation builder",
            VALIDATION_SERVICE_NAME
        ))),
    }
}
