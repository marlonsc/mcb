//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Java language processor for AST-based code chunking.

use crate::language::common::{
    AST_NODE_INTERFACE_DECLARATION, CHUNK_SIZE_JAVA, TS_NODE_CLASS_DECLARATION,
    TS_NODE_METHOD_DECLARATION,
};

crate::impl_simple_language_processor!(
    JavaProcessor,
    language = tree_sitter_java::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_JAVA,
    max_depth = 3,
    nodes = [
        TS_NODE_METHOD_DECLARATION,
        TS_NODE_CLASS_DECLARATION,
        AST_NODE_INTERFACE_DECLARATION
    ]
);
