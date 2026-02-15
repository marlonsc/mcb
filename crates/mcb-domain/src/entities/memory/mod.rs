//! Memory domain entities
//!
//! Includes observations, execution history, and memory search results.

mod error_pattern;
mod execution;
mod origin_context;
mod quality_gate;
mod search;
mod session;

pub use super::observation::{Observation, ObservationMetadata, ObservationType};
pub use error_pattern::{ErrorPattern, ErrorPatternCategory, ErrorPatternMatch};
pub use execution::{ExecutionMetadata, ExecutionType};
pub use origin_context::OriginContext;
pub use quality_gate::{QualityGateResult, QualityGateStatus};
pub use search::{MemoryFilter, MemorySearchIndex, MemorySearchResult};
pub use session::SessionSummary;
