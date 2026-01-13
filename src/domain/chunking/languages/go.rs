//! Go language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_GO;

crate::define_language_processor! {
    GoProcessor,
    tree_sitter_go::LANGUAGE,
    chunk_size: CHUNK_SIZE_GO,
    doc: "Go language processor with function and type extraction.",
    rules: [
        {
            node_types: ["function_declaration", "method_declaration", "type_declaration", "struct_type"],
            min_length: 35,
            min_lines: 2,
            max_depth: 3,
            priority: 8,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^func ", r"^type "],
}
