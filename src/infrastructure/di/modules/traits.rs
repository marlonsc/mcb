//! Submodule Trait Interfaces
//!
//! These traits define the interfaces for domain-specific DI modules.
//! Concrete implementations can be swapped out for testing or different environments.
//!
//! Note: These traits only extend `HasComponent<T>`, not `Module`.
//! The `Module` trait is automatically implemented by the `module!` macro.

use shaku::HasComponent;

use crate::adapters::http_client::HttpClientProvider;
use crate::application::admin::AdminService;
use crate::domain::ports::{
    ChunkRepository, ChunkingOrchestratorInterface, CodeChunker, ContextServiceInterface,
    EmbeddingProvider, IndexingOperationsInterface, IndexingServiceInterface,
    PerformanceMetricsInterface, SearchRepository, SearchServiceInterface, SnapshotProvider,
    SyncProvider, VectorStoreProvider,
};
use crate::infrastructure::auth::AuthServiceInterface;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::events::EventBusProvider;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;

/// Adapters module trait - external adapters like HTTP clients, providers, and repositories
pub trait AdaptersModule:
    HasComponent<dyn HttpClientProvider>
    + HasComponent<dyn EmbeddingProvider>
    + HasComponent<dyn VectorStoreProvider>
    + HasComponent<dyn ChunkRepository>
    + HasComponent<dyn SearchRepository>
{
}

/// Infrastructure module trait - core infrastructure services
pub trait InfrastructureModule:
    HasComponent<dyn SystemMetricsCollectorInterface>
    + HasComponent<dyn ServiceProviderInterface>
    + HasComponent<dyn EventBusProvider>
    + HasComponent<dyn AuthServiceInterface>
    + HasComponent<dyn SnapshotProvider>
    + HasComponent<dyn SyncProvider>
{
}

/// Server module trait - MCP server components
pub trait ServerModule:
    HasComponent<dyn PerformanceMetricsInterface> + HasComponent<dyn IndexingOperationsInterface>
{
}

/// Admin module trait - admin service with dependencies
pub trait AdminModule: HasComponent<dyn AdminService> {}

/// Application module trait - business logic services
pub trait ApplicationModule:
    HasComponent<dyn ContextServiceInterface>
    + HasComponent<dyn SearchServiceInterface>
    + HasComponent<dyn IndexingServiceInterface>
    + HasComponent<dyn ChunkingOrchestratorInterface>
    + HasComponent<dyn CodeChunker>
{
}

// ============================================================================
// Future Module Traits (v0.3.0+)
// ============================================================================

/// Analysis module trait - code complexity and technical debt detection (v0.3.0+)
///
/// Placeholder trait for future analysis capabilities including:
/// - Code complexity metrics (cyclomatic, cognitive)
/// - Technical debt detection
/// - SATD (Self-Admitted Technical Debt) identification
#[cfg(feature = "analysis")]
pub trait AnalysisModule: Send + Sync {}

/// Quality module trait - quality gates and assessment (v0.5.0+)
///
/// Placeholder trait for future quality capabilities including:
/// - Quality gate definitions and enforcement
/// - Code quality metrics
/// - Quality trend analysis
#[cfg(feature = "quality")]
pub trait QualityModule: Send + Sync {}

/// Git module trait - git operations and repository analysis (v0.5.0+)
///
/// Placeholder trait for future git integration including:
/// - Repository operations
/// - Commit history analysis
/// - Branch management
#[cfg(feature = "git")]
pub trait GitModule: Send + Sync {}
