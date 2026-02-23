//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Generic CRUD adapter module.

pub mod adapter;
pub mod mapper;
/// Adapter resolver
pub mod resolver;
pub mod unified;

pub use adapter::EntityCrudAdapter;
pub use resolver::resolve_adapter;
