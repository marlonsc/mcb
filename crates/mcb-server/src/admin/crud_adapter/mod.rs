//! Generic CRUD adapter module.

pub mod adapter;
pub mod mapper;
pub mod unified;
pub mod utils;

pub use adapter::EntityCrudAdapter;

use crate::admin::handlers::AdminState;
use mapper::slug_to_resource;
use unified::UnifiedEntityCrudAdapter;

/// Resolves a CRUD adapter for the given entity slug from `AdminState`.
#[must_use]
pub fn resolve_adapter(slug: &str, state: &AdminState) -> Option<Box<dyn EntityCrudAdapter>> {
    let handlers = state.tool_handlers.as_ref()?;
    let (resource, parent_field) = slug_to_resource(slug)?;

    Some(Box::new(UnifiedEntityCrudAdapter {
        resource,
        parent_field,
        handlers: handlers.clone(),
    }))
}
