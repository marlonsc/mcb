//! Administrative interfaces for system management and monitoring.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

/// Dashboard/analytics query ports.
pub mod dashboard;
/// Indexing operation tracking ports.
pub mod indexing;
/// Provider admin interfaces (embedding, vector store, language).
pub mod provider_admin;
/// Validation operation tracking ports.
pub mod validation;

// Re-exports for canonical access via `ports::admin::{...}`
pub use dashboard::{
    AgentSessionStats, DailyCount, DashboardQueryPort, MonthlyCount, ToolCallCount,
};
pub use indexing::{IndexingOperation, IndexingOperationStatus, IndexingOperationsInterface};
pub use provider_admin::{
    EmbeddingAdminInterface, LanguageAdminInterface, ProviderInfo, VectorStoreAdminInterface,
};
pub use validation::{
    ValidationOperation, ValidationOperationResult, ValidationOperationsInterface,
    ValidationStatus, ValidatorJobRunner,
};
