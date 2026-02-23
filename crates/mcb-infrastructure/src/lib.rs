//! # Infrastructure Layer
//!
//! Cross-cutting technical concerns that support the application and domain layers.
//!
//! This layer provides shared technical capabilities used across the entire system.
//! All adapters/providers are in mcb-providers crate, accessed via linkme registry.
//!
//! **Documentation**: [`docs/modules/infrastructure.md`](../../../docs/modules/infrastructure.md) |
//! **DI Architecture**: [`ADR-029`](../../../docs/adr/029-hexagonal-architecture-dill.md),
//! [`ADR-023`](../../../docs/adr/023-inventory-to-linkme-migration.md)
//!
//! ## Module Categories
//!
//! ### Security & Authentication
//! | Module | Description |
//! | -------- | ------------- |
//! | [`crypto`] | AES-GCM encryption, secure key generation |
//!
//! ### Data & Storage
//! | Module | Description |
//! | -------- | ------------- |
//! | [`cache`] | Moka/Redis caching with TTL and namespaces |
//!
//! ### Configuration & DI
//! | Module | Description |
//! | -------- | ------------- |
//! | [`config`] | TOML configuration with hot-reload |
//! | [`di`] | Handle-based dependency injection |
//! | [`constants`] | Centralized configuration constants |
//!
//! ### Observability
//! | Module | Description |
//! | -------- | ------------- |
//! | [`health`] | Health check endpoints |
//! | [`logging`] | Structured logging with tracing |
//!
//! ### Routing & Selection
//! | Module | Description |
//! | -------- | ------------- |
//! | [`routing`] | Provider routing and selection |

// Clippy allows for complex patterns in infrastructure code

// Core infrastructure modules
#[macro_use]
pub(crate) mod macros;
pub mod cache;
pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error_ext;
pub mod health;
pub mod logging;
pub mod project;
#[allow(missing_docs)]
pub mod provider_linker;
pub mod routing;
pub mod services;
pub mod utils;
pub mod validation;

pub mod infrastructure;
pub use error_ext::ErrorContext;
pub use utils::TimedOperation;
