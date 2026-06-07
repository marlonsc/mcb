//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Go language processor for AST-based code chunking.

use crate::language::common::CHUNK_SIZE_GO;
use mcb_domain::constants::ast::{TS_NODE_FUNCTION_DECLARATION, TS_NODE_METHOD_DECLARATION};

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
