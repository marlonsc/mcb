//! Go language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_GO, LanguageConfig, NodeExtractionRule, TS_NODE_FUNCTION_DECLARATION,
    TS_NODE_METHOD_DECLARATION,
};

/// Go language processor.
pub struct GoProcessor {
    processor: BaseProcessor,
}

impl GoProcessor {
    /// Create a new Go language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_go::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_string(),
                    TS_NODE_METHOD_DECLARATION.to_string(),
                    "type_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_chunk_size(CHUNK_SIZE_GO);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::impl_delegating_language_processor!(GoProcessor, processor);
