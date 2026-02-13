//! C# language processor for AST-based code chunking.
// TODO(qlty): Found 62 lines of similar code in 3 locations (mass = 159)

use crate::language::common::{
    AST_NODE_INTERFACE_DECLARATION, BaseProcessor, CHUNK_SIZE_CSHARP, LanguageConfig,
    NodeExtractionRule, TS_NODE_CLASS_DECLARATION, TS_NODE_METHOD_DECLARATION,
};

/// C# language processor.
pub struct CSharpProcessor {
    processor: BaseProcessor,
}

impl CSharpProcessor {
    /// Create a new C# language processor with default rules.
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_c_sharp::LANGUAGE.into())
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
            .with_chunk_size(CHUNK_SIZE_CSHARP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::impl_delegating_language_processor!(CSharpProcessor, processor);
