//! Language processor trait and implementations
//!
//! This module defines the LanguageProcessor trait that provides a common interface
//! for language-specific chunking logic.

use crate::chunking::config::LanguageConfig;
use crate::core::types::{CodeChunk, Language};

/// Trait for language-specific processing
pub trait LanguageProcessor {
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

    /// Extract chunks using fallback method
    fn extract_chunks_fallback(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk>;

    /// Get the language instance
    fn get_language(&self) -> tree_sitter::Language {
        (self.config().ts_language_fn)()
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
            let traverser = crate::chunking::traverser::AstTraverser::new(
                &self.config().extraction_rules,
                language,
            )
            .with_max_chunks(75); // Limit chunks per file
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

            // Sort by priority descending, then by start_line ascending
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

    fn extract_chunks_fallback(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        crate::chunking::fallback::GenericFallbackChunker::new(self.config())
            .chunk_with_patterns(content, file_name, language)
    }
}

/// Macro to implement LanguageProcessor for concrete processors
macro_rules! impl_language_processor {
    ($processor:ty) => {
        impl LanguageProcessor for $processor {
            fn config(&self) -> &LanguageConfig {
                &self.processor.config()
            }

            fn extract_chunks_with_tree_sitter(
                &self,
                tree: &tree_sitter::Tree,
                content: &str,
                file_name: &str,
                language: &Language,
            ) -> Vec<CodeChunk> {
                use crate::chunking::traverser::AstTraverser;

                let mut chunks = Vec::new();
                let mut cursor = tree.walk();

                if cursor.goto_first_child() {
                    let traverser = AstTraverser::new(&self.config().extraction_rules, language)
                        .with_max_chunks(75); // Limit chunks per file
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

                    // Sort by priority descending, then by start_line ascending
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

            fn extract_chunks_fallback(
                &self,
                content: &str,
                file_name: &str,
                language: &Language,
            ) -> Vec<CodeChunk> {
                crate::chunking::fallback::GenericFallbackChunker::new(self.config())
                    .chunk_with_patterns(content, file_name, language)
            }
        }
    };
}

// Export the macro for use in language-specific modules
pub(crate) use impl_language_processor;
