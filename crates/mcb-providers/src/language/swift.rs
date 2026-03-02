//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Swift language processor for AST-based code chunking.

use mcb_utils::constants::ast::{TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION};
use mcb_utils::constants::lang::CHUNK_SIZE_SWIFT;

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
