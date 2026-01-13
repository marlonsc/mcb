//! Java language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_JAVA;

crate::define_language_processor! {
    JavaProcessor,
    tree_sitter_java::LANGUAGE,
    chunk_size: CHUNK_SIZE_JAVA,
    doc: "Java language processor with method and class extraction.",
    rules: [
        {
            node_types: ["method_declaration", "class_declaration", "interface_declaration", "constructor_declaration"],
            min_length: 40,
            min_lines: 3,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^public .*\(.*\)", r"^private .*\(.*\)", r"^protected .*\(.*\)", r"^class ", r"^interface "],
}
