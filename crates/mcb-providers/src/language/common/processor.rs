//! Language processor trait and base implementation
//!
//! Defines the LanguageProcessor trait that provides a common interface
//! for language-specific chunking logic.

use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

use super::config::LanguageConfig;
use super::traverser::AstTraverser;

/// Trait for language-specific processing
///
/// # Example
///
/// ```no_run
/// use mcb_providers::language::common::LanguageProcessor;
///
/// // Parse code with tree-sitter
/// // let mut parser = tree_sitter::Parser::new();
/// // parser.set_language(processor.get_language())?;
/// // let tree = parser.parse(content, None)
/// //     .ok_or_else(|| anyhow::anyhow!("Failed to parse content"))?;
/// //
/// // Extract chunks using AST
/// // let chunks = processor.extract_chunks_with_tree_sitter(&tree, content, "main.rs", &Language::Rust);
/// ```
pub trait LanguageProcessor: Send + Sync {
    /// Get language configuration
    fn config(&self) -> &LanguageConfig;

    /// Extract chunks using tree-sitter
    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk>;

    /// Get the language instance
    fn get_language(&self) -> tree_sitter::Language {
        self.config().get_language()
    }
}

/// Base processor struct that holds configuration
#[derive(Debug)]
pub struct BaseProcessor {
    config: LanguageConfig,
}

impl BaseProcessor {
    /// Create a new base processor with configuration
    pub fn new(config: LanguageConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &LanguageConfig {
        &self.config
    }
}

impl LanguageProcessor for BaseProcessor {
    fn config(&self) -> &LanguageConfig {
        &self.config
    }

    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let mut cursor = tree.walk();

        if cursor.goto_first_child() {
            let traverser =
                AstTraverser::new(&self.config().extraction_rules, language).with_max_chunks(75);
            traverser.traverse_and_extract(&mut cursor, content, file_name, 0, &mut chunks);
        }

        // Sort chunks by priority (highest first) and then by line number
        chunks.sort_by(|a, b| {
            let a_priority = a
                .metadata
                .get("priority")
                .and_then(|p| p.as_i64())
                .unwrap_or(0);
            let b_priority = b
                .metadata
                .get("priority")
                .and_then(|p| p.as_i64())
                .unwrap_or(0);

            b_priority
                .cmp(&a_priority)
                .then(a.start_line.cmp(&b.start_line))
        });

        // Keep only top priority chunks if we have too many
        if chunks.len() > 50 {
            chunks.truncate(50);
        }

        chunks
    }
}

#[macro_export]

macro_rules! impl_delegating_language_processor {
    ($processor_ty:ty, $inner_field:ident) => {
        impl Default for $processor_ty {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::language::common::LanguageProcessor for $processor_ty {
            fn config(&self) -> &$crate::language::common::LanguageConfig {
                self.$inner_field.config()
            }

            fn extract_chunks_with_tree_sitter(
                &self,
                tree: &tree_sitter::Tree,
                content: &str,
                file_name: &str,
                language: &mcb_domain::value_objects::Language,
            ) -> Vec<mcb_domain::entities::CodeChunk> {
                self.$inner_field
                    .extract_chunks_with_tree_sitter(tree, content, file_name, language)
            }
        }
    };
}

#[macro_export]

macro_rules! impl_simple_language_processor {
    (
        $processor_ty:ident,
        language = $language:expr,
        chunk_size = $chunk_size:expr,
        max_depth = $max_depth:expr,
        nodes = [$($node_type:expr),+ $(,)?]
    ) => {


        pub struct $processor_ty {
            processor: $crate::language::common::BaseProcessor,
        }


        impl $processor_ty {

            pub fn new() -> Self {
                let config = $crate::language::common::LanguageConfig::new($language)
                    .with_rules(vec![$crate::language::common::NodeExtractionRule {
                        node_types: vec![$($node_type.to_string()),+],
                        min_length: 30,
                        min_lines: 2,
                        max_depth: $max_depth,
                        priority: 5,
                        include_context: true,
                    }])
                    .with_chunk_size($chunk_size);

                Self {
                    processor: $crate::language::common::BaseProcessor::new(config),
                }
            }
        }

        $crate::impl_delegating_language_processor!($processor_ty, processor);
    };
}
