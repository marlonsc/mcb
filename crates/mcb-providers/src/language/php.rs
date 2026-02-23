//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! PHP language processor for AST-based code chunking.

use crate::language::common::{
    CHUNK_SIZE_PHP, TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DEFINITION,
    TS_NODE_METHOD_DECLARATION,
};

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
