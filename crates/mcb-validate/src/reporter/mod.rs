//! Validation Report Generation
//!
//! Generates reports in multiple formats:
//! - JSON for CI integration
//! - Human-readable for terminal output
//! - CI summary for GitHub Actions annotations

mod report;
mod summary;

pub use report::{Reporter, ValidationReport};
pub use summary::ValidationSummary;
