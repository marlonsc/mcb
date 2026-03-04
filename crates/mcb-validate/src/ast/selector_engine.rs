use std::path::{Path, PathBuf};

use tree_sitter::{Language, Parser};

use crate::ast::{AstDecoder, AstQueryBuilder, QueryCondition};
use crate::rules::yaml_loader::{AstSelector, ValidatedRule};

/// A single match produced by the AST selector engine for a validated rule.
#[derive(Debug, Clone)]
pub struct AstSelectorMatch {
    /// Path to the file where the match occurred.
    pub file_path: PathBuf,
    /// One-based line number of the match.
    pub line: usize,
}

/// Engine that executes AST selectors (tree-sitter-based) against source files.
pub struct AstSelectorEngine;

impl AstSelectorEngine {
    /// Runs the given rule's AST selectors against `file` and returns all matches.
    #[must_use]
    pub fn execute(rule: &ValidatedRule, file: &Path) -> Vec<AstSelectorMatch> {
        if rule.selectors.is_empty() {
            return Vec::new();
        }

        let source = match crate::run_context::ValidationRunContext::active()
            .and_then(|ctx| ctx.read_cached(file).ok())
        {
            Some(cached) => cached.to_string(),
            None => match std::fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    mcb_domain::warn!(
                        "validate",
                        "Failed to read file for AST selector validation",
                        &format!("file = {}, error = {:?}", file.display(), e)
                    );
                    return Vec::new();
                }
            },
        };

        Self::execute_on_source(rule, file, &source)
    }

    #[must_use]
    pub(crate) fn execute_on_source(
        rule: &ValidatedRule,
        file: &Path,
        source: &str,
    ) -> Vec<AstSelectorMatch> {
        if rule.selectors.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        for selector in &rule.selectors {
            if !Self::selector_matches_file_language(selector, file) {
                continue;
            }

            let Some(root) = Self::parse_ast(selector, source) else {
                continue;
            };

            let mut query_builder = AstQueryBuilder::new(&selector.language, &selector.node_type)
                .message(&Self::violation_message(rule))
                .severity(&rule.severity);

            if let Some(pattern) = &selector.pattern {
                query_builder = query_builder.with_condition(QueryCondition::NameMatches {
                    pattern: pattern.clone(),
                });
            }

            let query = query_builder.build();
            let violations = query.execute(&root);
            matches.extend(violations.into_iter().map(|violation| AstSelectorMatch {
                file_path: file.to_path_buf(),
                line: violation.node.span.start.line,
            }));
        }

        matches
    }

    fn parse_ast(selector: &AstSelector, source: &str) -> Option<crate::ast::AstNode> {
        let language = Self::tree_sitter_language(&selector.language)?;
        let mut parser = Parser::new();
        parser.set_language(&language).ok()?;
        let tree = parser.parse(source, None)?;
        Some(AstDecoder::decode_tree(&tree, source))
    }

    fn tree_sitter_language(language: &str) -> Option<Language> {
        use mcb_domain::ports::validation::LanguageId;

        let id = LanguageId::from_name(language)?;
        #[allow(clippy::wildcard_enum_match_arm)]
        match id {
            LanguageId::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            LanguageId::Python => Some(tree_sitter_python::LANGUAGE.into()),
            LanguageId::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            LanguageId::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            LanguageId::Go => Some(tree_sitter_go::LANGUAGE.into()),
            LanguageId::Java => Some(tree_sitter_java::LANGUAGE.into()),
            LanguageId::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            _other_lang => None,
        }
    }

    fn selector_matches_file_language(selector: &AstSelector, file: &Path) -> bool {
        use mcb_domain::ports::validation::LanguageId;

        let expected_lang = LanguageId::from_name(&selector.language);
        let file_ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_lang = LanguageId::from_extension(file_ext);

        match (expected_lang, file_lang) {
            (Some(e), Some(f)) => e == f,
            _ => false,
        }
    }

    fn violation_message(rule: &ValidatedRule) -> String {
        rule.message
            .clone()
            .unwrap_or_else(|| format!("[{}] AST selector match: {}", rule.id, rule.description))
    }
}
