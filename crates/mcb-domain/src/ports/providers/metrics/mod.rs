//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
mod metric_defs;
mod provider;

pub use metric_defs::{MetricLabels, MetricsError, MetricsResult};
pub use provider::MetricsProvider;
