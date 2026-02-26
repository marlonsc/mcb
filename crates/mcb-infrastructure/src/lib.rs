//! # Infrastructure Layer
//!
//! Cross-cutting technical concerns that support the application and domain layers.
//!
//! This layer provides shared technical capabilities used across the entire system.
//! All adapters/providers are in mcb-providers crate, accessed via linkme registry.
//!
//! **Documentation**: [`docs/modules/infrastructure.md`](../../../docs/modules/infrastructure.md) |
//! **DI Architecture**: [`ADR-029`](../../../docs/adr/archive/superseded-029-hexagonal-architecture-dill.md),
//! [`ADR-023`](../../../docs/adr/023-inventory-to-linkme-migration.md)
//!
//! ## Module Categories
//!
//! ### Security & Authentication
//! | Module | Description |
//! | -------- | ------------- |
//! | [`crypto`] | AES-GCM encryption, secure key generation |
//!
//! ### Configuration & DI
//! | Module | Description |
//! | -------- | ------------- |
//! | [`config`] | YAML configuration with hot-reload |
//! | [`constants`] | Centralized configuration constants |
//!
//! ### Routing & Selection
//! | Module | Description |
//! | -------- | ------------- |
//! | [`routing`] | Provider routing and selection |

// Clippy allows for complex patterns in infrastructure code

#[macro_use]
pub(crate) mod macros;
pub mod config;
pub mod constants;
pub mod crypto;
pub mod infrastructure;

pub mod logging;
pub mod project;
pub mod routing;
pub mod services;
pub mod utils;
pub mod validation;

mod exports;
pub use exports::*;
pub mod events;
/// Repository trait re-exports for infrastructure consumers.
pub mod repositories;
/// DI resolution context for service factory functions.
pub mod resolution_context;
