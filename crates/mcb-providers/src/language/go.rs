//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Go language processor for AST-based code chunking.

use mcb_utils::constants::ast::{TS_NODE_FUNCTION_DECLARATION, TS_NODE_METHOD_DECLARATION};
use mcb_utils::constants::lang::CHUNK_SIZE_GO;

crate::impl_simple_language_processor!(
    GoProcessor,
    language = tree_sitter_go::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_GO,
    max_depth = 2,
    nodes = [
        TS_NODE_FUNCTION_DECLARATION,
        TS_NODE_METHOD_DECLARATION,
        "type_declaration"
    ]
);
