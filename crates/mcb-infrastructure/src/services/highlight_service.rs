//! Highlight Service - Agnóstico code highlighting using tree-sitter
//!
//! Provides trait-based interface for syntax highlighting across multiple languages.
//! Uses tree-sitter for accurate, efficient parsing and highlighting.
//!
//! Designed for multiple renderers: Web (Phase 8a), TUI (Phase 9), etc.

use std::sync::Arc;

use anyhow::Context;
use mcb_domain::ports::browse::{HighlightError, HighlightServiceInterface};
use mcb_domain::value_objects::browse::{HighlightCategory, HighlightSpan, HighlightedCode};
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use crate::constants::highlight::HIGHLIGHT_NAMES;

/// Language-specific highlighting configuration
struct HighlightLanguageConfig {
    name: &'static str,
    language: Language,
    highlights_query: &'static str,
}

impl HighlightLanguageConfig {
    fn new(name: &'static str, language: Language, highlights_query: &'static str) -> Self {
        Self {
            name,
            language,
            highlights_query,
        }
    }
}

/// Maps tree-sitter highlight names to our category enum.
pub fn map_highlight_to_category(name: &str) -> HighlightCategory {
    match name {
        "keyword" => HighlightCategory::Keyword,
        "string" => HighlightCategory::String,
        "comment" => HighlightCategory::Comment,
        "function" => HighlightCategory::Function,
        "type" => HighlightCategory::Type,
        "variable" => HighlightCategory::Variable,
        "number" => HighlightCategory::Number,
        "operator" => HighlightCategory::Operator,
        "punctuation" => HighlightCategory::Punctuation,
        _ => HighlightCategory::Other,
    }
}

/// Concrete highlight service implementation using tree-sitter
pub struct HighlightServiceImpl {
    highlighter: Arc<tokio::sync::Mutex<Highlighter>>,
}

impl HighlightServiceImpl {
    pub fn new() -> Self {
        Self {
            highlighter: Arc::new(tokio::sync::Mutex::new(Highlighter::new())),
        }
    }

    /// Get language configuration for supported languages
    fn get_language_config(language: &str) -> Result<HighlightLanguageConfig, HighlightError> {
        let normalized = language.trim().to_lowercase();

        match normalized.as_str() {
            "rust" => Ok(HighlightLanguageConfig::new(
                "rust",
                tree_sitter_rust::LANGUAGE.into(),
                tree_sitter_rust::HIGHLIGHTS_QUERY,
            )),
            "python" => Ok(HighlightLanguageConfig::new(
                "python",
                tree_sitter_python::LANGUAGE.into(),
                tree_sitter_python::HIGHLIGHTS_QUERY,
            )),
            "javascript" | "js" => Ok(HighlightLanguageConfig::new(
                "javascript",
                tree_sitter_javascript::LANGUAGE.into(),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
            )),
            "typescript" | "ts" => Ok(HighlightLanguageConfig::new(
                "typescript",
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            )),
            "tsx" => Ok(HighlightLanguageConfig::new(
                "tsx",
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            )),
            "go" => Ok(HighlightLanguageConfig::new(
                "go",
                tree_sitter_go::LANGUAGE.into(),
                tree_sitter_go::HIGHLIGHTS_QUERY,
            )),
            "java" => Ok(HighlightLanguageConfig::new(
                "java",
                tree_sitter_java::LANGUAGE.into(),
                tree_sitter_java::HIGHLIGHTS_QUERY,
            )),
            "c" => Ok(HighlightLanguageConfig::new(
                "c",
                tree_sitter_c::LANGUAGE.into(),
                tree_sitter_c::HIGHLIGHT_QUERY,
            )),
            "cpp" | "c++" => Ok(HighlightLanguageConfig::new(
                "cpp",
                tree_sitter_cpp::LANGUAGE.into(),
                tree_sitter_cpp::HIGHLIGHT_QUERY,
            )),
            "ruby" => Ok(HighlightLanguageConfig::new(
                "ruby",
                tree_sitter_ruby::LANGUAGE.into(),
                tree_sitter_ruby::HIGHLIGHTS_QUERY,
            )),
            "php" => Ok(HighlightLanguageConfig::new(
                "php",
                tree_sitter_php::LANGUAGE_PHP.into(),
                tree_sitter_php::HIGHLIGHTS_QUERY,
            )),
            "swift" => Ok(HighlightLanguageConfig::new(
                "swift",
                tree_sitter_swift::LANGUAGE.into(),
                tree_sitter_swift::HIGHLIGHTS_QUERY,
            )),
            _ => Err(HighlightError::UnsupportedLanguage(language.to_string())),
        }
    }

    /// Create highlight configuration from language config
    fn create_highlight_config(
        lang_config: HighlightLanguageConfig,
    ) -> Result<HighlightConfiguration, HighlightError> {
        let mut config = HighlightConfiguration::new(
            lang_config.language,
            lang_config.name,
            lang_config.highlights_query,
            "",
            "",
        )
        .map_err(|e| HighlightError::ConfigurationError(e.to_string()))?;

        config.configure(&HIGHLIGHT_NAMES);
        Ok(config)
    }

    /// Highlight code using tree-sitter
    fn highlight_code_internal(
        &self,
        code: &str,
        language: &str,
    ) -> Result<HighlightedCode, HighlightError> {
        if code.is_empty() {
            return Ok(HighlightedCode {
                original: code.to_string(),
                spans: vec![],
                language: language.to_string(),
            });
        }

        let config_err = |op: &str, e: HighlightError| {
            HighlightError::ConfigurationError(format!("failed to {op} for '{language}': {e}"))
        };

        let lang_config = Self::get_language_config(language)
            .context("get language config")
            .map_err(|e| HighlightError::ConfigurationError(e.to_string()))?;

        let config = Self::create_highlight_config(lang_config)
            .map_err(|e| config_err("create highlight config", e))?;

        let mut highlighter = self.highlighter.blocking_lock();

        let highlights = highlighter
            .highlight(&config, code.as_bytes(), None, |_: &str| None)
            .map_err(|e| HighlightError::HighlightingFailed(e.to_string()))?;

        let mut spans = Vec::new();
        let mut position = 0;
        let mut open_spans: Vec<(usize, &str)> = Vec::new();

        for event in highlights {
            match event {
                Ok(HighlightEvent::Source { end, .. }) => {
                    position = end;
                }
                Ok(HighlightEvent::HighlightStart(Highlight(highlight))) => {
                    if let Some(class_name) = HIGHLIGHT_NAMES.get(highlight) {
                        open_spans.push((position, class_name));
                    }
                }
                Ok(HighlightEvent::HighlightEnd) => {
                    if let Some((start, class_name)) = open_spans.pop() {
                        let category = map_highlight_to_category(class_name);
                        spans.push(HighlightSpan {
                            start,
                            end: position,
                            category,
                        });
                    }
                }
                Err(e) => {
                    return Err(HighlightError::HighlightingFailed(e.to_string()));
                }
            }
        }

        Ok(HighlightedCode {
            original: code.to_string(),
            spans,
            language: language.to_string(),
        })
    }
}

impl Default for HighlightServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl HighlightServiceInterface for HighlightServiceImpl {
    async fn highlight(&self, code: &str, language: &str) -> mcb_domain::Result<HighlightedCode> {
        let code = code.to_string();
        let language = language.to_string();
        let highlighter = Arc::clone(&self.highlighter);

        let result = tokio::task::spawn_blocking(move || {
            let service = HighlightServiceImpl { highlighter };
            service.highlight_code_internal(&code, &language)
        })
        .await
        .map_err(|e| HighlightError::HighlightingFailed(format!("Blocking task failed: {}", e)))?;

        result.map_err(mcb_domain::Error::from)
    }
}

/// Convert HighlightedCode to HTML with CSS classes
pub fn convert_highlighted_code_to_html(highlighted: &HighlightedCode) -> String {
    if highlighted.original.is_empty() {
        return String::new();
    }

    let mut html = String::new();
    let mut last_end = 0;

    let mut sorted_spans = highlighted.spans.clone();
    sorted_spans.sort_by_key(|s| s.start);

    for span in sorted_spans {
        if last_end < span.start {
            let text = &highlighted.original[last_end..span.start];
            html.push_str(&html_escape(text));
        }

        let class = category_to_css_class(span.category);
        let text = &highlighted.original[span.start..span.end];
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            class,
            html_escape(text)
        ));

        last_end = span.end;
    }

    if last_end < highlighted.original.len() {
        let text = &highlighted.original[last_end..];
        html.push_str(&html_escape(text));
    }

    html
}

fn category_to_css_class(category: HighlightCategory) -> &'static str {
    match category {
        HighlightCategory::Keyword => "hl-keyword",
        HighlightCategory::String => "hl-string",
        HighlightCategory::Comment => "hl-comment",
        HighlightCategory::Function => "hl-function",
        HighlightCategory::Variable => "hl-variable",
        HighlightCategory::Type => "hl-type",
        HighlightCategory::Number => "hl-number",
        HighlightCategory::Operator => "hl-operator",
        HighlightCategory::Punctuation => "hl-punctuation",
        HighlightCategory::Other => "hl-other",
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
