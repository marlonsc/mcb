//! Re-export highlight functions from consolidat ed handlers module
//!
//! This module provides backward compatibility and serves as a public API
//! for syntax highlighting. All implementation is in handlers::highlight_service.

pub use crate::handlers::highlight_service::{highlight_chunks, highlight_code};
