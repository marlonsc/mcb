//! Dependency Injection System — Manual Composition Root + linkme + Handle
//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Two-layer DI combining compile-time discovery with runtime flexibility:
//!
//! - **linkme registry**: `#[distributed_slice]` for compile-time provider discovery
//! - **Handle pattern**: `RwLock<Arc<dyn T>>` for runtime provider switching
//!
//! ```text
//! linkme (compile-time)       AppContext (manual wiring)     Handle-based
//! ─────────────────────       ─────────────────────────      ────────────
//! EMBEDDING_PROVIDERS    →    Resolver → init_app()     →   Handle (RwLock)
//! (distributed slices)                                           ↓
//!                                                          AdminService
//!                                                         (switch via API)
//! ```
//!
//! ## Key Principles
//!
//! - **Manual Composition Root**: `init_app()` in `bootstrap.rs` wires all services
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//! - **Compile-time Discovery**: linkme `#[distributed_slice]` registers providers
//! - **Runtime Switching**: Providers can be changed via admin API + Handle
//! - **Testability**: Default local providers (`FastEmbed`, `EdgeVec`) enable isolated testing
//!
//! ## For Consumer Crates (mcb-server tests, golden tests, etc.)
//!
//! - `create_test_dependencies()` — isolated repos + shared providers from `AppContext`
//! - `create_memory_repository()` / `create_memory_repository_with_executor()` — standalone repos
//! - `create_vcs_entity_repository()` — standalone VCS repo
//!
//! ## Config-Driven Initialization
//!
//! All providers initialize from `config/default.toml` + overrides.
//! Use `TestConfigBuilder` for test-specific overrides.
//! **NEVER import from `mcb_providers` directly outside this crate.**

pub mod admin;
pub mod bootstrap;
pub mod database_resolver;
pub mod handle;
pub mod handles;
pub mod modules;
pub mod provider_resolvers;
pub mod repositories;
pub mod resolver;
pub mod test_factory;
#[allow(missing_docs)]
pub mod vcs;

pub use admin::{
    AdminService, CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface,
    EmbeddingAdminService, LanguageAdminInterface, LanguageAdminService, ProviderInfo,
    ProviderResolver, VectorStoreAdminInterface, VectorStoreAdminService,
};
pub use bootstrap::*;
pub use handle::Handle;
pub use handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
pub use modules::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
pub use provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, EventBusProviderResolver,
    FileSystemProviderResolver, LanguageProviderResolver, TaskRunnerProviderResolver,
    VcsProviderResolver, VectorStoreProviderResolver,
};
pub use repositories::{
    create_memory_repository, create_memory_repository_with_executor, create_vcs_entity_repository,
};
pub use resolver::{
    AvailableProviders, ResolvedProviders, list_available_providers, resolve_providers,
};
pub use test_factory::create_test_dependencies;
pub use vcs::default_vcs_provider;
