//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use super::adapter::EntityCrudAdapter;
use super::mapper::slug_to_resource;
use super::unified::UnifiedEntityCrudAdapter;
use crate::admin::handlers::AdminState;

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
