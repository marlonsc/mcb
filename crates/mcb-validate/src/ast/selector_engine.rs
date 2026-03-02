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

        let source = match std::fs::read_to_string(file) {
            Ok(source) => source,
            Err(e) => {
                mcb_domain::warn!(
                    "validate",
                    "Failed to read file for AST selector validation",
                    &format!("file = {}, error = {:?}", file.display(), e)
                );
                return Vec::new();
            }
        };

        let mut matches = Vec::new();
        for selector in &rule.selectors {
            if !Self::selector_matches_file_language(selector, file) {
                continue;
            }

            let Some(root) = Self::parse_ast(selector, &source) else {
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
        match language.to_ascii_lowercase().as_str() {
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "javascript" | "js" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "typescript" | "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            "java" => Some(tree_sitter_java::LANGUAGE.into()),
            "c" => Some(tree_sitter_c::LANGUAGE.into()),
            "cpp" | "c++" => Some(tree_sitter_cpp::LANGUAGE.into()),
            _ => None,
        }
    }

    fn selector_matches_file_language(selector: &AstSelector, file: &Path) -> bool {
        let expected = selector.language.to_ascii_lowercase();
        let from_ext = file
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);

        match from_ext.as_deref() {
            Some("rs") => expected == "rust",
            Some("py") => expected == "python",
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => {
                expected == "javascript" || expected == "js"
            }
            Some("ts") | Some("tsx") | Some("mts") | Some("cts") => {
                expected == "typescript" || expected == "ts"
            }
            Some("go") => expected == "go",
            Some("java") => expected == "java",
            Some("c") | Some("h") => expected == "c" || expected == "cpp" || expected == "c++",
            Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") | Some("hxx") => {
                expected == "cpp" || expected == "c++"
            }
            _ => false,
        }
    }

    fn violation_message(rule: &ValidatedRule) -> String {
        rule.message
            .clone()
            .unwrap_or_else(|| format!("[{}] AST selector match: {}", rule.id, rule.description))
    }
}
