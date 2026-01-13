//! C# language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_CSHARP;

crate::define_language_processor! {
    CSharpProcessor,
    tree_sitter_c_sharp::LANGUAGE,
    chunk_size: CHUNK_SIZE_CSHARP,
    doc: "C# language processor with method and class extraction.",
    rules: [
        {
            node_types: ["method_declaration", "class_declaration", "interface_declaration", "property_declaration"],
            min_length: 40,
            min_lines: 3,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^public .*\(.*\)", r"^private .*\(.*\)", r"^protected .*\(.*\)", r"^class ", r"^interface "],
}
