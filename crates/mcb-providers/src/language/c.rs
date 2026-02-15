//! C language processor for AST-based code chunking.

use crate::language::common::{
    AST_NODE_STRUCT_SPECIFIER, CHUNK_SIZE_C, TS_NODE_FUNCTION_DEFINITION,
};

crate::impl_simple_language_processor!(
    CProcessor,
    language = tree_sitter_c::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_C,
    max_depth = 2,
    nodes = [TS_NODE_FUNCTION_DEFINITION, AST_NODE_STRUCT_SPECIFIER]
);
