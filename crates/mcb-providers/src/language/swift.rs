//! Swift language processor for AST-based code chunking.

use crate::language::common::{
    CHUNK_SIZE_SWIFT, TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION,
};

crate::impl_simple_language_processor!(
    SwiftProcessor,
    language = tree_sitter_swift::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_SWIFT,
    max_depth = 3,
    nodes = [
        TS_NODE_FUNCTION_DECLARATION,
        TS_NODE_CLASS_DECLARATION,
        "struct_declaration",
        "protocol_declaration"
    ]
);
