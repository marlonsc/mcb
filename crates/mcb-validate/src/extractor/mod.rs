//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Code fact extraction from source files.
//!
//! This module provides AST-based extraction of code "facts" (modules, structs,
//! functions, imports, etc.) from Rust source files using `rust-code-analysis`.

pub mod fact;
pub mod rust_extractor;

pub use fact::{Fact, FactType, Location};
pub use rust_extractor::RustExtractor;
