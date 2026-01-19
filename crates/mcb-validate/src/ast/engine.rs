//! AST Engine Module
//!
//! Core AST processing engine and parser trait.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::core::AstParseResult;
use super::languages;
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

        // Register language parsers
        parsers.insert(
            "rust".to_string(),
            Arc::new(Mutex::new(languages::RustParser::new())),
        );
        parsers.insert(
            "python".to_string(),
            Arc::new(Mutex::new(languages::PythonParser::new())),
        );
        parsers.insert(
            "javascript".to_string(),
            Arc::new(Mutex::new(languages::JavaScriptParser::new())),
        );
        parsers.insert(
            "typescript".to_string(),
            Arc::new(Mutex::new(languages::TypeScriptParser::new())),
        );
        parsers.insert(
            "go".to_string(),
            Arc::new(Mutex::new(languages::GoParser::new())),
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

    pub fn detect_language(&self, path: &Path) -> Option<&str> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "tsx" => Some("typescript"),
            "go" => Some("go"),
            _ => None,
        }
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}
