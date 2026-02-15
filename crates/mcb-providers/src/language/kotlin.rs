//! Kotlin language processor for AST-based code chunking.

use crate::language::common::{
    CHUNK_SIZE_KOTLIN, TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION,
};

crate::impl_simple_language_processor!(
    KotlinProcessor,
    language = tree_sitter_kotlin_ng::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_KOTLIN,
    max_depth = 3,
    nodes = [
        TS_NODE_FUNCTION_DECLARATION,
        TS_NODE_CLASS_DECLARATION,
        "object_declaration"
    ]
);
