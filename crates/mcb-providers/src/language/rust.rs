//! Rust language processor for AST-based code chunking.

use crate::language::common::{BaseProcessor, CHUNK_SIZE_RUST, LanguageConfig, NodeExtractionRule};

/// Rust language processor with comprehensive AST extraction rules.
pub struct RustProcessor {
    processor: BaseProcessor,
}

impl RustProcessor {
    /// Create a new Rust language processor
    #[must_use]
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())
            .with_rules(Self::extraction_rules())
            .with_chunk_size(CHUNK_SIZE_RUST);

        Self {
            processor: BaseProcessor::new(config),
        }
    }

    fn extraction_rules() -> Vec<NodeExtractionRule> {
        vec![
            NodeExtractionRule::primary(&[
                "function_item",
                "struct_item",
                "enum_item",
                "impl_item",
                "trait_item",
            ]),
            NodeExtractionRule::secondary(&[
                "mod_item",
                "macro_definition",
                "const_item",
                "static_item",
            ]),
            NodeExtractionRule::tertiary(&["type_item", "use_declaration"]),
        ]
    }
}

crate::impl_delegating_language_processor!(RustProcessor, processor);
