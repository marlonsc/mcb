//! Python language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_PYTHON, LanguageConfig, NodeExtractionRule,
    TS_NODE_FUNCTION_DEFINITION,
};

/// Python language processor with function and class extraction.
pub struct PythonProcessor {
    processor: BaseProcessor,
}

impl PythonProcessor {
    /// Create a new Python language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_python::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DEFINITION.to_string(),
                    "class_definition".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_chunk_size(CHUNK_SIZE_PYTHON);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::impl_delegating_language_processor!(PythonProcessor, processor);
