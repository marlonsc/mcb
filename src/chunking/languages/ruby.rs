//! Ruby language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_RUBY;

crate::define_language_processor! {
    RubyProcessor,
    tree_sitter_ruby::LANGUAGE,
    chunk_size: CHUNK_SIZE_RUBY,
    doc: "Ruby language processor with method, class, and module extraction.",
    rules: [
        {
            node_types: ["method", "class", "module", "singleton_method"],
            min_length: 30,
            min_lines: 2,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
        {
            node_types: ["block", "lambda"],
            min_length: 20,
            min_lines: 2,
            max_depth: 2,
            priority: 5,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^def ", r"^class ", r"^module ", r"^attr_"],
}
