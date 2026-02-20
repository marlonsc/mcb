//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! C++ language processor for AST-based code chunking.

use crate::language::common::{
    AST_NODE_STRUCT_SPECIFIER, CHUNK_SIZE_CPP, TS_NODE_FUNCTION_DEFINITION,
};

crate::impl_simple_language_processor!(
    CppProcessor,
    language = tree_sitter_cpp::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_CPP,
    max_depth = 3,
    nodes = [
        TS_NODE_FUNCTION_DEFINITION,
        "class_specifier",
        AST_NODE_STRUCT_SPECIFIER
    ]
);
