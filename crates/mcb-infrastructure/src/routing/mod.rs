//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Provider Routing Infrastructure
//!
//! Provides intelligent routing and selection of backend providers
//! based on health status, cost, quality, and operational requirements.
//!
//! ## Components
//!
//! - [`DefaultProviderRouter`] - Production router with health tracking
//!
//! ## Usage via DI
//!
//! ```text
//! // Providers are obtained via DI container
//! // let router: Arc<dyn ProviderRouter> = container.resolve();
//! // let provider = router.select_embedding_provider(&context).await?;
//! ```

mod health;
mod router;

// Re-export for DI registration
pub use health::{HealthMonitor, InMemoryHealthMonitor};
pub use router::DefaultProviderRouter;
