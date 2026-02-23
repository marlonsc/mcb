//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
mod provider;
mod types;

pub use provider::MetricsAnalysisProvider;
pub use types::{FileMetrics, FunctionMetrics, HalsteadMetrics};
