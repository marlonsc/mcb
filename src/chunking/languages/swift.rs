//! Swift language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_SWIFT;

crate::define_language_processor! {
    SwiftProcessor,
    tree_sitter_swift::LANGUAGE,
    chunk_size: CHUNK_SIZE_SWIFT,
    doc: "Swift language processor with function, class, and protocol extraction.",
    rules: [
        {
            node_types: ["function_declaration", "class_declaration", "struct_declaration", "protocol_declaration", "enum_declaration"],
            min_length: 35,
            min_lines: 2,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
        {
            node_types: ["computed_property", "subscript_declaration", "initializer_declaration"],
            min_length: 25,
            min_lines: 2,
            max_depth: 2,
            priority: 7,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^func ", r"^class ", r"^struct ", r"^protocol ", r"^enum ", r"^extension "],
}
