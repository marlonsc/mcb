//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! PHP language processor for AST-based code chunking.

use mcb_utils::constants::ast::{
    TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DEFINITION, TS_NODE_METHOD_DECLARATION,
};
use mcb_utils::constants::lang::CHUNK_SIZE_PHP;

crate::impl_simple_language_processor!(
    PhpProcessor,
    language = tree_sitter_php::LANGUAGE_PHP.into(),
    chunk_size = CHUNK_SIZE_PHP,
    max_depth = 2,
    nodes = [
        TS_NODE_FUNCTION_DEFINITION,
        TS_NODE_METHOD_DECLARATION,
        TS_NODE_CLASS_DECLARATION
    ]
);
