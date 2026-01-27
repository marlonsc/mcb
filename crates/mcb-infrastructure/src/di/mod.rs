//! Dependency Injection System - dill IoC + Handle-based Pattern + linkme Registry
//!
//! This module implements dependency injection using:
//! - **dill Catalog**: IoC container for service resolution
//! - **Handle-based pattern**: Runtime-swappable provider handles with RwLock
//! - **linkme registry**: Compile-time discovery of external providers
//!
//! ## Architecture
//!
//! ```text
//! linkme (compile-time)     dill Catalog (runtime)     Handle-based
//! ─────────────────────     ─────────────────────      ────────────
//! EMBEDDING_PROVIDERS  →    Resolver → add_value() →   Handle (RwLock)
//! (list of factories)                                       ↓
//!                                                     AdminService
//!                                                    (switch via API)
//! ```
//!
//! ## Key Principles
//!
//! - **IoC Container**: dill Catalog manages service lifecycle
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//! - **Composition Root**: Services composed in catalog.rs build_catalog()
//! - **Runtime Switching**: Providers can be changed via admin API
//! - **Testability**: Null providers enable isolated testing

pub mod admin;
pub mod bootstrap;
pub mod catalog;
pub mod dispatch;
pub mod handle;
pub mod handles;
pub mod modules;
pub mod provider_resolvers;
pub mod resolver;

pub use admin::{
    AdminService, CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface,
    EmbeddingAdminService, LanguageAdminInterface, LanguageAdminService, ProviderInfo,
    ProviderResolver, VectorStoreAdminInterface, VectorStoreAdminService,
};
pub use bootstrap::*;
pub use catalog::build_catalog;
pub use dispatch::*;
pub use handle::Handle;
pub use handles::{
    CacheHandleExt, CacheProviderHandle, EmbeddingHandleExt, EmbeddingProviderHandle,
    LanguageProviderHandle, VectorStoreProviderHandle,
};
pub use modules::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
pub use provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
pub use resolver::{
    AvailableProviders, ResolvedProviders, list_available_providers, resolve_providers,
};
