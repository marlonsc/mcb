//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Common utilities for language chunking providers
//!
//! This module contains shared code used by all language-specific processors.

pub mod config;
pub mod constants;
pub mod detection;
pub mod engine;
pub mod processor;
pub mod traverser;

// Re-export commonly used types
pub use config::{LanguageConfig, NodeExtractionRule, NodeExtractionRuleBuilder};
pub use constants::*;
pub use processor::{BaseProcessor, LanguageProcessor};
pub use traverser::AstTraverser;
