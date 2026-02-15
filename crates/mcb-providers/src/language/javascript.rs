//! JavaScript/TypeScript language processor for AST-based code chunking.

use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

use crate::language::common::{
    AST_NODE_INTERFACE_DECLARATION, BaseProcessor, CHUNK_SIZE_JAVASCRIPT, LanguageConfig,
    LanguageProcessor, NodeExtractionRule, TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION,
};

/// JavaScript/TypeScript language processor.
pub struct JavaScriptProcessor {
    processor: BaseProcessor,
}

impl JavaScriptProcessor {
    /// Create a new JavaScript/TypeScript language processor
    #[must_use]
    pub fn new(is_typescript: bool) -> Self {
        let ts_language = if is_typescript {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        } else {
            tree_sitter_javascript::LANGUAGE.into()
        };

        let config = LanguageConfig::new(ts_language)
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_owned(),
                    TS_NODE_CLASS_DECLARATION.to_owned(),
                    "method_definition".to_owned(),
                    "arrow_function".to_owned(),
                    AST_NODE_INTERFACE_DECLARATION.to_owned(),
                    "type_alias_declaration".to_owned(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_chunk_size(CHUNK_SIZE_JAVASCRIPT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl Default for JavaScriptProcessor {
    fn default() -> Self {
        Self::new(false)
    }
}

impl LanguageProcessor for JavaScriptProcessor {
    fn config(&self) -> &LanguageConfig {
        self.processor.config()
    }

    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        self.processor
            .extract_chunks_with_tree_sitter(tree, content, file_name, language)
    }
}
