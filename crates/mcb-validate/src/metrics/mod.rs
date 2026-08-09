//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Code Metrics Module
//!
//! Provides code complexity metrics analysis using rust-code-analysis (RCA).
//!
//! ## Supported Metrics
//!
//! - **Cyclomatic Complexity**: Number of linearly independent paths
//! - **Cognitive Complexity**: Measures how difficult code is to understand
//! - **Halstead metrics**: Volume, Difficulty, Effort
//! - **Maintainability Index**: Overall maintainability score (0-100)
//! - **LOC metrics**: SLOC, PLOC, LLOC, CLOC, BLANK
//! - **NOM, NARGS, NEXITS**: Method count, argument count, exit points
//!
//! ## Supported Languages
//!
//! Rust, Python, JavaScript, TypeScript, Java, C, C++, Kotlin (via `RcaAnalyzer`)

mod rca_analyzer;
mod thresholds;
mod violation;

pub use self::violation::MetricViolation;
pub use rca_analyzer::{RcaAnalyzer, RcaFunctionMetrics, RcaMetrics};
pub use thresholds::{MetricThreshold, MetricThresholds, MetricType};
