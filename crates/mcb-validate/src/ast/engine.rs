//! AST Engine Module
//!
//! Core AST processing engine and parser trait.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rust_code_analysis::guess_language;

use super::core::AstParseResult;
use super::languages::TreeSitterParser;
use super::query::AstQuery;
use crate::Result;

/// Language-specific AST parser
pub trait AstParser: Send + Sync {
    fn language(&self) -> &'static str;
    fn parse_file(&mut self, path: &Path) -> Result<AstParseResult>;
    fn parse_content(&mut self, content: &str, filename: &str) -> Result<AstParseResult>;
}

/// Unified AST engine for multi-language analysis
pub struct AstEngine {
    parsers: HashMap<String, Arc<Mutex<dyn AstParser>>>,
    queries: HashMap<String, Vec<AstQuery>>,
}

impl AstEngine {
    pub fn new() -> Self {
        let mut parsers: HashMap<String, Arc<Mutex<dyn AstParser>>> = HashMap::new();

        parsers.insert(
            "rust".to_string(),
            Arc::new(Mutex::new(TreeSitterParser::rust())),
        );
        parsers.insert(
            "python".to_string(),
            Arc::new(Mutex::new(TreeSitterParser::python())),
        );
        parsers.insert(
            "javascript".to_string(),
            Arc::new(Mutex::new(TreeSitterParser::javascript())),
        );
        parsers.insert(
            "typescript".to_string(),
            Arc::new(Mutex::new(TreeSitterParser::typescript())),
        );
        parsers.insert(
            "go".to_string(),
            Arc::new(Mutex::new(TreeSitterParser::go())),
        );

        Self {
            parsers,
            queries: HashMap::new(),
        }
    }

    pub fn register_query(&mut self, rule_id: String, query: AstQuery) {
        self.queries.entry(rule_id).or_default().push(query);
    }

    pub fn get_parser(&self, language: &str) -> Option<&Arc<Mutex<dyn AstParser>>> {
        self.parsers.get(language)
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.parsers
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Detect programming language from file path using rust-code-analysis
    pub fn detect_language(&self, path: &Path) -> Option<&str> {
        // Try RCA's guess_language if file exists
        if let Ok(source) = std::fs::read(path) {
            let (lang, _) = guess_language(&source, path);
            if let Some(name) = lang.map(|l| l.get_name()) {
                return match name {
                    "rust" => Some("rust"),
                    "python" => Some("python"),
                    "javascript" | "mozjs" => Some("javascript"),
                    "typescript" | "tsx" => Some("typescript"),
                    _ => None,
                };
            }
        }

        // Fallback to extension-based detection
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("rust"),
                "py" => Some("python"),
                "js" | "mjs" | "cjs" => Some("javascript"),
                "ts" | "tsx" => Some("typescript"),
                "go" => Some("go"),
                _ => None,
            })
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}
