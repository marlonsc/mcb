//! Language-specific AST parsers using Tree-sitter
//!
//! Provides a unified parser that handles multiple programming languages,
//! converting Tree-sitter AST to the internal format used for validation queries.

use std::path::Path;
use tree_sitter::{Language, Parser};

use super::{AstParseResult, AstParser};
use crate::Result;

/// Tree-sitter based AST parser supporting multiple languages
pub struct TreeSitterParser {
    parser: Parser,
    language_name: &'static str,
}

impl TreeSitterParser {
    /// Create a parser for the specified language
    fn new(grammar: Language, language_name: &'static str) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&grammar)
            .expect("Failed to load grammar");

        Self {
            parser,
            language_name,
        }
    }

    /// Create a Rust parser
    pub fn rust() -> Self {
        Self::new(tree_sitter_rust::LANGUAGE.into(), "rust")
    }

    /// Create a Python parser
    pub fn python() -> Self {
        Self::new(tree_sitter_python::LANGUAGE.into(), "python")
    }

    /// Create a JavaScript parser
    pub fn javascript() -> Self {
        Self::new(tree_sitter_javascript::LANGUAGE.into(), "javascript")
    }

    /// Create a TypeScript/TSX parser
    pub fn typescript() -> Self {
        Self::new(tree_sitter_typescript::LANGUAGE_TSX.into(), "typescript")
    }

    /// Create a Go parser
    pub fn go() -> Self {
        Self::new(tree_sitter_go::LANGUAGE.into(), "go")
    }
}

impl AstParser for TreeSitterParser {
    fn language(&self) -> &'static str {
        self.language_name
    }

    fn parse_file(&mut self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&mut self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree =
            self.parser
                .parse(content, None)
                .ok_or_else(|| crate::ValidationError::Parse {
                    file: filename.into(),
                    message: format!("Failed to parse {} code", self.language_name),
                })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser() {
        let mut parser = TreeSitterParser::rust();
        assert_eq!(parser.language(), "rust");

        let code = r#"
fn hello_world() {
    println!("Hello, World!");
}
"#;
        let result = parser.parse_content(code, "test.rs").unwrap();
        assert_eq!(result.root.kind, "source_file");
        assert!(!result.root.children.is_empty());
    }

    #[test]
    fn test_python_parser() {
        let mut parser = TreeSitterParser::python();
        assert_eq!(parser.language(), "python");

        let code = r#"
def hello_world():
    print("Hello, World!")
"#;
        let result = parser.parse_content(code, "test.py").unwrap();
        assert_eq!(result.root.kind, "module");
        assert!(!result.root.children.is_empty());
    }

    #[test]
    fn test_javascript_parser() {
        let parser = TreeSitterParser::javascript();
        assert_eq!(parser.language(), "javascript");
    }

    #[test]
    fn test_typescript_parser() {
        let parser = TreeSitterParser::typescript();
        assert_eq!(parser.language(), "typescript");
    }

    #[test]
    fn test_go_parser() {
        let parser = TreeSitterParser::go();
        assert_eq!(parser.language(), "go");
    }
}
