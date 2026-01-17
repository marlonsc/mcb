//! Dependency Injection layer
//!
//! Uses Shaku for dependency injection to wire together infrastructure components.
//! This follows Clean Architecture principles by providing a composition root
//! that resolves dependencies and creates the application container.
//!
//! **ARCHITECTURE**: This module contains ONLY wiring logic.
//! Admin service implementations live in crate::di::admin.
//! Provider adapters are in mcb-providers crate.

pub mod admin;
pub mod bootstrap;
pub mod dispatch;
pub mod factory;
pub mod modules;
pub mod registry;

pub use admin::*;
pub use bootstrap::*;
pub use dispatch::*;
pub use factory::*;
pub use modules::*;
pub use registry::*;
