//! Swift language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_SWIFT, LanguageConfig, NodeExtractionRule, TS_NODE_CLASS_DECLARATION,
    TS_NODE_FUNCTION_DECLARATION,
};

/// Swift language processor.
pub struct SwiftProcessor {
    processor: BaseProcessor,
}

impl SwiftProcessor {
    /// Create a new Swift language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_swift::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_string(),
                    TS_NODE_CLASS_DECLARATION.to_string(),
                    "struct_declaration".to_string(),
                    "protocol_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_chunk_size(CHUNK_SIZE_SWIFT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::impl_delegating_language_processor!(SwiftProcessor, processor);
