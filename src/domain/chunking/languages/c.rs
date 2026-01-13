//! C language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_C;

crate::define_language_processor! {
    CProcessor,
    tree_sitter_c::LANGUAGE,
    chunk_size: CHUNK_SIZE_C,
    doc: "C language processor with function and struct extraction.",
    rules: [
        {
            node_types: ["function_definition", "struct_specifier", "enum_specifier"],
            min_length: 30,
            min_lines: 2,
            max_depth: 2,
            priority: 8,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(", r"^struct ", r"^enum ", r"^typedef "],
}
