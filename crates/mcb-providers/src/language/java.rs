//! Java language processor for AST-based code chunking.

use crate::language::common::{
    AST_NODE_INTERFACE_DECLARATION, BaseProcessor, CHUNK_SIZE_JAVA, LanguageConfig,
    NodeExtractionRule, TS_NODE_CLASS_DECLARATION, TS_NODE_METHOD_DECLARATION,
};

/// Java language processor.
pub struct JavaProcessor {
    processor: BaseProcessor,
}

impl JavaProcessor {
    /// Create a new Java language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_java::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_METHOD_DECLARATION.to_string(),
                    TS_NODE_CLASS_DECLARATION.to_string(),
                    AST_NODE_INTERFACE_DECLARATION.to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_chunk_size(CHUNK_SIZE_JAVA);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::impl_delegating_language_processor!(JavaProcessor, processor);
