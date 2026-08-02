//! Intelligent chunking engine
//!
//! Provides the main `IntelligentChunker` that orchestrates language-specific
//! chunking using tree-sitter and fallback methods.

mod chunker;
mod processors;
mod universal_provider;

pub use chunker::IntelligentChunker;
pub use universal_provider::*;
