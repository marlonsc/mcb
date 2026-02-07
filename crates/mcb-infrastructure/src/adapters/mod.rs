//! Infrastructure Adapters
//!
//! Provides adapter interfaces for DI integration.
//! Following Clean Architecture: adapters implement domain interfaces.

pub mod metrics_analysis;
pub mod validation;

pub use metrics_analysis::NullMetricsProvider;
pub use validation::NullValidationProvider;
