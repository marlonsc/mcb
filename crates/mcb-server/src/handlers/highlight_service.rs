//! Highlight Service - Agnóstico code highlighting using tree-sitter
//!
//! Provides trait-based interface for syntax highlighting across multiple languages.
//! Uses tree-sitter for accurate, efficient parsing and highlighting.
//!
//! Designed for multiple renderers: Web (Phase 8a), TUI (Phase 9), etc.

use super::browse_service::{HighlightCategory, HighlightSpan, HighlightedCode};
use std::sync::Arc;
use thiserror::Error;
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

/// Highlight service errors
#[derive(Debug, Error)]
pub enum HighlightError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub type Result<T> = std::result::Result<T, HighlightError>;

/// Language-specific highlighting configuration
struct LanguageConfig {
    name: &'static str,
    language: Language,
    highlights_query: &'static str,
}

impl LanguageConfig {
    fn new(name: &'static str, language: Language, highlights_query: &'static str) -> Self {
        Self {
            name,
            language,
            highlights_query,
        }
    }
}

/// Highlight categories mapping
const HIGHLIGHT_NAMES: [&str; 13] = [
    "keyword",
    "function",
    "string",
    "comment",
    "type",
    "variable",
    "constant",
    "operator",
    "attribute",
    "number",
    "punctuation",
    "property",
    "tag",
];

/// Maps tree-sitter highlight names to our category enum
fn map_highlight_to_category(name: &str) -> HighlightCategory {
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

/// Highlight service trait (agnóstico interface)
pub trait HighlightService: Send + Sync {
    /// Highlight code with given language
    ///
    /// Returns structured highlight spans with byte offsets.
    /// Falls back to empty spans if highlighting fails.
    fn highlight(
        &self,
        code: &str,
        language: &str,
    ) -> impl std::future::Future<Output = Result<HighlightedCode>> + Send;
}

/// Concrete highlight service implementation using tree-sitter
pub struct HighlightServiceImpl {
    highlighter: Arc<std::sync::Mutex<Highlighter>>,
}

impl HighlightServiceImpl {
    pub fn new() -> Self {
        Self {
            highlighter: Arc::new(std::sync::Mutex::new(Highlighter::new())),
        }
    }

    /// Get language configuration for supported languages
    fn get_language_config(language: &str) -> Option<LanguageConfig> {
        let normalized = language.trim().to_lowercase();

        match normalized.as_str() {
            "rust" => Some(LanguageConfig::new(
                "rust",
                tree_sitter_rust::LANGUAGE.into(),
                tree_sitter_rust::HIGHLIGHTS_QUERY,
            )),
            "python" => Some(LanguageConfig::new(
                "python",
                tree_sitter_python::LANGUAGE.into(),
                tree_sitter_python::HIGHLIGHTS_QUERY,
            )),
            "javascript" | "js" => Some(LanguageConfig::new(
                "javascript",
                tree_sitter_javascript::LANGUAGE.into(),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
            )),
            "typescript" | "ts" => Some(LanguageConfig::new(
                "typescript",
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            )),
            "tsx" => Some(LanguageConfig::new(
                "tsx",
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            )),
            "go" => Some(LanguageConfig::new(
                "go",
                tree_sitter_go::LANGUAGE.into(),
                tree_sitter_go::HIGHLIGHTS_QUERY,
            )),
            "java" => Some(LanguageConfig::new(
                "java",
                tree_sitter_java::LANGUAGE.into(),
                tree_sitter_java::HIGHLIGHTS_QUERY,
            )),
            "c" => Some(LanguageConfig::new(
                "c",
                tree_sitter_c::LANGUAGE.into(),
                tree_sitter_c::HIGHLIGHT_QUERY,
            )),
            "cpp" | "c++" => Some(LanguageConfig::new(
                "cpp",
                tree_sitter_cpp::LANGUAGE.into(),
                tree_sitter_cpp::HIGHLIGHT_QUERY,
            )),
            "ruby" => Some(LanguageConfig::new(
                "ruby",
                tree_sitter_ruby::LANGUAGE.into(),
                tree_sitter_ruby::HIGHLIGHTS_QUERY,
            )),
            "php" => Some(LanguageConfig::new(
                "php",
                tree_sitter_php::LANGUAGE_PHP.into(),
                tree_sitter_php::HIGHLIGHTS_QUERY,
            )),
            "swift" => Some(LanguageConfig::new(
                "swift",
                tree_sitter_swift::LANGUAGE.into(),
                tree_sitter_swift::HIGHLIGHTS_QUERY,
            )),
            _ => None,
        }
    }

    /// Create highlight configuration from language config
    fn create_highlight_config(lang_config: LanguageConfig) -> Result<HighlightConfiguration> {
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
    fn highlight_code_internal(&self, code: &str, language: &str) -> Result<HighlightedCode> {
        if code.is_empty() {
            return Ok(HighlightedCode {
                original: code.to_string(),
                spans: vec![],
                language: language.to_string(),
            });
        }

        let lang_config = Self::get_language_config(language)
            .ok_or_else(|| HighlightError::UnsupportedLanguage(language.to_string()))?;

        let config = Self::create_highlight_config(lang_config)?;

        let mut highlighter = self
            .highlighter
            .lock()
            .map_err(|_| HighlightError::HighlightingFailed("Mutex lock failed".to_string()))?;

        let highlights = highlighter
            .highlight(&config, code.as_bytes(), None, |_| None)
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

impl HighlightService for HighlightServiceImpl {
    fn highlight(
        &self,
        code: &str,
        language: &str,
    ) -> impl std::future::Future<Output = Result<HighlightedCode>> + Send {
        let code = code.to_string();
        let language = language.to_string();
        let highlighter = Arc::clone(&self.highlighter);

        async move {
            tokio::task::spawn_blocking(move || {
                let service = HighlightServiceImpl { highlighter };
                service.highlight_code_internal(&code, &language)
            })
            .await
            .map_err(|e| HighlightError::HighlightingFailed(e.to_string()))?
        }
    }
}

/// Public function to highlight code and return HTML with syntax highlighting
///
/// This is the main entry point for syntax highlighting. It uses the internal
/// HighlightServiceImpl to perform tree-sitter based highlighting and converts
/// the result to HTML with CSS classes.
///
/// # Arguments
///
/// * `code` - The source code to highlight
/// * `language` - The programming language (e.g., "rust", "python", "javascript")
///
/// # Returns
///
/// HTML string with syntax highlighting applied via CSS classes (hl-keyword, hl-string, etc.)
/// Returns empty string for empty input or unsupported languages.
///
/// # Example
///
/// ```ignore
/// let html = highlight_code("fn main() {}", "rust");
/// assert!(html.contains("<span class=\"hl-keyword\">fn</span>"));
/// ```
pub fn highlight_code(code: &str, language: &str) -> String {
    if code.is_empty() {
        return String::new();
    }

    let code_owned = code.to_string();
    let language_owned = language.to_string();
    let service = HighlightServiceImpl::new();

    // Use blocking call since this is a sync function
    match std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { service.highlight(&code_owned, &language_owned).await })
    })
    .join()
    {
        Ok(Ok(highlighted)) => convert_highlighted_code_to_html(&highlighted),
        Ok(Err(_)) => {
            // Fallback to HTML-escaped plain text for unsupported languages
            html_escape(code)
        }
        Err(_) => {
            // Thread panic fallback
            html_escape(code)
        }
    }
}

/// Public function to highlight code chunks and return HTML
///
/// Highlights multiple code chunks and returns HTML with syntax highlighting.
/// Useful for highlighting multiple code blocks from the same file.
///
/// # Arguments
///
/// * `chunks` - Vector of (code, language) tuples
///
/// # Returns
///
/// Vector of HTML strings with syntax highlighting applied
pub fn highlight_chunks(chunks: Vec<(&str, &str)>) -> Vec<String> {
    chunks
        .into_iter()
        .map(|(code, language)| highlight_code(code, language))
        .collect()
}

/// Convert HighlightedCode to HTML with CSS classes
fn convert_highlighted_code_to_html(highlighted: &HighlightedCode) -> String {
    if highlighted.original.is_empty() {
        return String::new();
    }

    let mut html = String::new();
    let mut last_end = 0;

    // Sort spans by start position for proper nesting
    let mut sorted_spans = highlighted.spans.clone();
    sorted_spans.sort_by_key(|s| s.start);

    for span in sorted_spans {
        // Add unspanned text before this span
        if last_end < span.start {
            let text = &highlighted.original[last_end..span.start];
            html.push_str(&html_escape(text));
        }

        // Add the highlighted span
        let class = category_to_css_class(span.category);
        let text = &highlighted.original[span.start..span.end];
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            class,
            html_escape(text)
        ));

        last_end = span.end;
    }

    // Add remaining unspanned text
    if last_end < highlighted.original.len() {
        let text = &highlighted.original[last_end..];
        html.push_str(&html_escape(text));
    }

    html
}

/// Map highlight category to CSS class name
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

/// HTML escape a string to prevent XSS
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_highlight_rust_keyword() {
        let service = HighlightServiceImpl::new();
        let code = "fn main() {}";
        let result = service
            .highlight(code, "rust")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "rust");
        // Should have some spans for keywords
        assert!(!result.spans.is_empty());
        // Check for keyword category
        assert!(
            result
                .spans
                .iter()
                .any(|s| s.category == HighlightCategory::Keyword)
        );
    }

    #[tokio::test]
    async fn test_highlight_python_number() {
        let service = HighlightServiceImpl::new();
        let code = "x = 1";
        let result = service
            .highlight(code, "python")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "python");
        // Should have some spans
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_javascript() {
        let service = HighlightServiceImpl::new();
        let code = "const x = 42;";
        let result = service
            .highlight(code, "javascript")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "javascript");
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_typescript() {
        let service = HighlightServiceImpl::new();
        let code = "let x: number = 42;";
        let result = service
            .highlight(code, "typescript")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "typescript");
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_go() {
        let service = HighlightServiceImpl::new();
        let code = "func main() {}";
        let result = service
            .highlight(code, "go")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "go");
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_java() {
        let service = HighlightServiceImpl::new();
        let code = "public class Main {}";
        let result = service
            .highlight(code, "java")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "java");
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_cpp() {
        let service = HighlightServiceImpl::new();
        let code = "int main() { return 0; }";
        let result = service
            .highlight(code, "cpp")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "cpp");
    }

    #[tokio::test]
    async fn test_highlight_ruby() {
        let service = HighlightServiceImpl::new();
        let code = "def hello; end";
        let result = service
            .highlight(code, "ruby")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "ruby");
    }

    #[tokio::test]
    async fn test_highlight_php() {
        let service = HighlightServiceImpl::new();
        let code = "<?php echo 'hello'; ?>";
        let result = service
            .highlight(code, "php")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "php");
    }

    #[tokio::test]
    async fn test_highlight_swift() {
        let service = HighlightServiceImpl::new();
        let code = "func main() {}";
        let result = service
            .highlight(code, "swift")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "swift");
    }

    #[tokio::test]
    async fn test_highlight_empty_code() {
        let service = HighlightServiceImpl::new();
        let result = service
            .highlight("", "rust")
            .await
            .expect("Failed to highlight");

        assert!(result.original.is_empty());
        assert!(result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_unsupported_language() {
        let service = HighlightServiceImpl::new();
        let result = service.highlight("code", "brainfuck").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HighlightError::UnsupportedLanguage(lang) => {
                assert_eq!(lang, "brainfuck");
            }
            _ => panic!("Expected UnsupportedLanguage error"),
        }
    }

    #[tokio::test]
    async fn test_highlight_fallback_to_plain_text() {
        let service = HighlightServiceImpl::new();
        // Use a language that will return empty spans as fallback
        let code = "some code";
        let result = service.highlight(code, "plaintext").await;

        // Should fail with unsupported language
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_highlight_performance_under_500ms() {
        let service = HighlightServiceImpl::new();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n".repeat(50);

        let start = Instant::now();
        let result = service
            .highlight(&code, "rust")
            .await
            .expect("Failed to highlight");
        let elapsed = start.elapsed();

        assert_eq!(result.language, "rust");
        assert!(
            elapsed.as_millis() < 500,
            "Highlighting took {}ms, expected < 500ms",
            elapsed.as_millis()
        );
    }

    #[tokio::test]
    async fn test_highlight_multiline_rust() {
        let service = HighlightServiceImpl::new();
        let code = r#"
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}
"#;
        let result = service
            .highlight(code, "rust")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "rust");
        // Multi-line code should have multiple spans
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_case_insensitive_language() {
        let service = HighlightServiceImpl::new();
        let code = "fn main() {}";

        let result_lower = service
            .highlight(code, "rust")
            .await
            .expect("Failed to highlight");

        let result_upper = service
            .highlight(code, "RUST")
            .await
            .expect("Failed to highlight");

        assert_eq!(result_lower.spans.len(), result_upper.spans.len());
    }

    #[tokio::test]
    async fn test_highlight_with_comments() {
        let service = HighlightServiceImpl::new();
        let code = r#"// This is a comment
fn main() {
    /* Multi-line
       comment */
    println!("Hello");
}
"#;
        let result = service
            .highlight(code, "rust")
            .await
            .expect("Failed to highlight");

        // Should have comment spans
        assert!(
            result
                .spans
                .iter()
                .any(|s| s.category == HighlightCategory::Comment)
        );
    }

    #[test]
    fn test_highlight_category_mapping() {
        assert_eq!(
            map_highlight_to_category("keyword"),
            HighlightCategory::Keyword
        );
        assert_eq!(
            map_highlight_to_category("string"),
            HighlightCategory::String
        );
        assert_eq!(
            map_highlight_to_category("comment"),
            HighlightCategory::Comment
        );
        assert_eq!(
            map_highlight_to_category("number"),
            HighlightCategory::Number
        );
        assert_eq!(
            map_highlight_to_category("unknown"),
            HighlightCategory::Other
        );
    }
}
