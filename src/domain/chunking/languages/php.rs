//! PHP language processor for AST-based code chunking.

use crate::infrastructure::constants::CHUNK_SIZE_PHP;

crate::define_language_processor! {
    PhpProcessor,
    tree_sitter_php::LANGUAGE_PHP,
    chunk_size: CHUNK_SIZE_PHP,
    doc: "PHP language processor with function, class, and trait extraction.",
    rules: [
        {
            node_types: ["function_definition", "method_declaration", "class_declaration", "interface_declaration", "trait_declaration"],
            min_length: 35,
            min_lines: 2,
            max_depth: 3,
            priority: 9,
            include_context: true,
        },
        {
            node_types: ["anonymous_function_creation_expression", "arrow_function"],
            min_length: 20,
            min_lines: 1,
            max_depth: 2,
            priority: 5,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^function ", r"^public function ", r"^private function ", r"^protected function ", r"^class ", r"^interface ", r"^trait "],
}
