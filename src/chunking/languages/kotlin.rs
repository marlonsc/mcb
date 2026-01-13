//! Kotlin language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_KOTLIN;

crate::define_language_processor! {
    KotlinProcessor,
    tree_sitter_kotlin_ng::LANGUAGE,
    chunk_size: CHUNK_SIZE_KOTLIN,
    doc: "Kotlin language processor with function, class, and object extraction.",
    rules: [
        {
            node_types: ["function_declaration", "class_declaration", "object_declaration", "interface_declaration"],
            min_length: 35,
            min_lines: 2,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
        {
            node_types: ["property_declaration", "companion_object", "anonymous_function", "lambda_literal"],
            min_length: 25,
            min_lines: 2,
            max_depth: 2,
            priority: 6,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^fun ", r"^class ", r"^object ", r"^interface ", r"^data class ", r"^sealed class "],
}
