//! C++ language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_CPP;

crate::define_language_processor! {
    CppProcessor,
    tree_sitter_cpp::LANGUAGE,
    chunk_size: CHUNK_SIZE_CPP,
    doc: "C++ language processor with class and template extraction.",
    rules: [
        {
            node_types: ["function_definition", "class_specifier", "struct_specifier", "template_declaration"],
            min_length: 40,
            min_lines: 3,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^template ", r"^class ", r"^struct ", r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\("],
}
