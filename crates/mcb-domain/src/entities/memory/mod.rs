//! Memory domain entities
//!
//! Includes observations, execution history, and memory search results.

mod error_pattern;
mod execution;
mod observation;
mod quality_gate;
mod search;
mod session;

pub use error_pattern::{ErrorPattern, ErrorPatternCategory, ErrorPatternMatch};
pub use execution::{ExecutionMetadata, ExecutionType};
pub use observation::{Observation, ObservationMetadata, ObservationType};
pub use quality_gate::{QualityGateResult, QualityGateStatus};
pub use search::{MemoryFilter, MemorySearchIndex, MemorySearchResult};
pub use session::SessionSummary;
