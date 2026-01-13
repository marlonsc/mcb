//! JavaScript/TypeScript language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_JAVASCRIPT;

crate::define_language_processor_with_param! {
    JavaScriptProcessor,
    language_fn: |lang: crate::domain::types::Language| -> tree_sitter::Language {
        match lang {
            crate::domain::types::Language::TypeScript => tree_sitter_typescript::LANGUAGE_TSX.into(),
            _ => tree_sitter_javascript::LANGUAGE.into(),
        }
    },
    chunk_size: CHUNK_SIZE_JAVASCRIPT,
    doc: "JavaScript/TypeScript language processor supporting both languages.",
    rules: [
        {
            node_types: ["function_declaration", "function", "class_declaration", "method_definition", "arrow_function"],
            min_length: 30,
            min_lines: 2,
            max_depth: 2,
            priority: 9,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^function ", r"^const .*=>\s*\{", r"^class "],
}
