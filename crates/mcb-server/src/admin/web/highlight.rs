//! Re-export highlight functions from infrastructure services module
//!
//! This module provides backward compatibility and serves as a public API
//! for syntax highlighting. All implementation is in mcb_infrastructure::services::highlight.

pub use mcb_infrastructure::services::highlight::{highlight_chunks, highlight_code};
