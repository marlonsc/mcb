//! Python language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_PYTHON;

crate::define_language_processor! {
    PythonProcessor,
    tree_sitter_python::LANGUAGE,
    chunk_size: CHUNK_SIZE_PYTHON,
    doc: "Python language processor with function and class extraction.",
    rules: [
        {
            node_types: ["function_definition", "class_definition"],
            min_length: 30,
            min_lines: 2,
            max_depth: 2,
            priority: 5,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^def ", r"^class "],
}
