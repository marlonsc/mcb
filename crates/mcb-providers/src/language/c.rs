//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! C language processor for AST-based code chunking.

use mcb_utils::constants::ast::{AST_NODE_STRUCT_SPECIFIER, TS_NODE_FUNCTION_DEFINITION};
use mcb_utils::constants::lang::CHUNK_SIZE_C;

crate::impl_simple_language_processor!(
    CProcessor,
    language = tree_sitter_c::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_C,
    max_depth = 2,
    nodes = [TS_NODE_FUNCTION_DEFINITION, AST_NODE_STRUCT_SPECIFIER]
);
