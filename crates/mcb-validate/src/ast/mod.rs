//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! AST Analysis Module
//!
//! Provides unified AST parsing and querying across multiple programming languages.
//! Uses rust-code-analysis (RCA) as the primary backend for parsing and traversal.
//!
//! # RCA Integration
//!
//! This module uses RCA's `action()`, `find()`, `guess_language()` directly:
//! - Language detection: `rust_code_analysis::guess_language()`
//! - Parsing: `rust_code_analysis::action::<Callback>(lang, source, path, None, cfg)`
//! - Node search: `rust_code_analysis::find(parser, &filters)`

pub mod core;
pub mod decoder;
pub mod query;
/// Shared RCA helpers — thin utilities over native `FuncSpace`/`SpaceKind` types.
pub mod rca_helpers;
/// Tree-sitter-based AST selector engine for rule-driven node matching.
pub mod selector_engine;
/// Tree-sitter query executor for running tree-sitter queries against source files.
pub mod tree_sitter_query_executor;
pub mod types;
pub mod unwrap_detector;

// Re-export public types and interfaces
pub use core::{AstNode, AstParseResult, Position, Span};

pub use decoder::AstDecoder;
pub use query::{AstQuery, AstQueryBuilder, AstQueryPatterns, QueryCondition};
pub use selector_engine::{AstSelectorEngine, AstSelectorMatch};
pub use tree_sitter_query_executor::{TreeSitterQueryExecutor, TreeSitterQueryMatch};
// Re-export RCA types for direct usage (NO wrappers)
pub use rust_code_analysis::{
    Callback, LANG, Node, ParserTrait, Search, action, find, guess_language,
};
pub use types::AstViolation;
pub use unwrap_detector::{UnwrapDetection, UnwrapDetector};
