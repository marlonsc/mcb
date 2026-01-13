//! Rust language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_RUST;

crate::define_language_processor! {
    RustProcessor,
    tree_sitter_rust::LANGUAGE,
    chunk_size: CHUNK_SIZE_RUST,
    doc: "Rust language processor with comprehensive AST extraction rules.",
    rules: [
        {
            node_types: ["function_item", "struct_item", "enum_item", "impl_item", "trait_item"],
            min_length: 40,
            min_lines: 2,
            max_depth: 4,
            priority: 10,
            include_context: true,
        },
        {
            node_types: ["mod_item", "macro_definition", "const_item", "static_item"],
            min_length: 25,
            min_lines: 1,
            max_depth: 3,
            priority: 5,
            include_context: false,
        },
        {
            node_types: ["type_item", "use_declaration"],
            min_length: 15,
            min_lines: 1,
            max_depth: 2,
            priority: 1,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^fn ", r"^struct ", r"^impl ", r"^pub fn ", r"^pub struct ", r"^enum ", r"^trait ", r"^mod ", r"^const ", r"^static "],
}
