//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Highlight Service Use Case
//!
//! # Overview
//! The `HighlightService` provides backend-agnostic syntax highlighting capabilities using
//! Tree-sitter. It parses source code into an abstract syntax tree (AST) to identify
//! tokens and apply semantic highlighting rules, independent of the final output format.
//!
//! # Responsibilities
//! - **Multi-Language Support**: Parsing and highlighting code for supported languages (Rust, Python, JS, etc.).
//! - **Tree-Sitter Integration**: Leveraging widely-used grammars for accurate syntax analysis.
//! - **Abstract Representation**: Producing a generic `HighlightedCode` structure (spans + categories)
//!   that can be rendered to HTML, ANSI, or other formats.

use std::sync::Arc;

use mcb_domain::ports::{HighlightError, HighlightServiceInterface};
use mcb_domain::registry::services::ServiceBuilder;
use mcb_domain::value_objects::browse::{HighlightSpan, HighlightedCode};
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use mcb_domain::value_objects::browse::{HIGHLIGHT_NAMES, map_highlight_to_category};

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

/// Concrete highlight service implementation using tree-sitter.
///
/// Manages a thread-safe `Highlighter` instance and specific language configurations
/// to perform efficient, on-demand syntax highlighting.
pub struct HighlightServiceImpl {
    highlighter: Arc<tokio::sync::Mutex<Highlighter>>,
}

impl HighlightServiceImpl {
    /// Creates a syntax highlight service with an internal tree-sitter highlighter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            highlighter: Arc::new(tokio::sync::Mutex::new(Highlighter::new())),
        }
    }

    fn get_language_config_dynamic(
        lang_id: mcb_domain::ports::validation::LanguageId,
    ) -> Option<Result<HighlightLanguageConfig, HighlightError>> {
        #[allow(clippy::wildcard_enum_match_arm)]
        match lang_id {
            mcb_domain::ports::validation::LanguageId::Python => {
                Some(Ok(HighlightLanguageConfig::new(
                    "python",
                    tree_sitter_python::LANGUAGE.into(),
                    tree_sitter_python::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::JavaScript => {
                Some(Ok(HighlightLanguageConfig::new(
                    "javascript",
                    tree_sitter_javascript::LANGUAGE.into(),
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::TypeScript => {
                Some(Ok(HighlightLanguageConfig::new(
                    "typescript",
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Tsx => {
                Some(Ok(HighlightLanguageConfig::new(
                    "tsx",
                    tree_sitter_typescript::LANGUAGE_TSX.into(),
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Ruby => {
                Some(Ok(HighlightLanguageConfig::new(
                    "ruby",
                    tree_sitter_ruby::LANGUAGE.into(),
                    tree_sitter_ruby::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Php => {
                Some(Ok(HighlightLanguageConfig::new(
                    "php",
                    tree_sitter_php::LANGUAGE_PHP.into(),
                    tree_sitter_php::HIGHLIGHTS_QUERY,
                )))
            }
            _unsupported_lang => None,
        }
    }

    fn get_language_config_static(
        lang_id: mcb_domain::ports::validation::LanguageId,
    ) -> Option<Result<HighlightLanguageConfig, HighlightError>> {
        #[allow(clippy::wildcard_enum_match_arm)]
        match lang_id {
            mcb_domain::ports::validation::LanguageId::Rust => {
                Some(Ok(HighlightLanguageConfig::new(
                    "rust",
                    tree_sitter_rust::LANGUAGE.into(),
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Go => {
                Some(Ok(HighlightLanguageConfig::new(
                    "go",
                    tree_sitter_go::LANGUAGE.into(),
                    tree_sitter_go::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Java => {
                Some(Ok(HighlightLanguageConfig::new(
                    "java",
                    tree_sitter_java::LANGUAGE.into(),
                    tree_sitter_java::HIGHLIGHTS_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::C => Some(Ok(HighlightLanguageConfig::new(
                "c",
                tree_sitter_c::LANGUAGE.into(),
                tree_sitter_c::HIGHLIGHT_QUERY,
            ))),
            mcb_domain::ports::validation::LanguageId::Cpp => {
                Some(Ok(HighlightLanguageConfig::new(
                    "cpp",
                    tree_sitter_cpp::LANGUAGE.into(),
                    tree_sitter_cpp::HIGHLIGHT_QUERY,
                )))
            }
            mcb_domain::ports::validation::LanguageId::Swift => {
                Some(Ok(HighlightLanguageConfig::new(
                    "swift",
                    tree_sitter_swift::LANGUAGE.into(),
                    tree_sitter_swift::HIGHLIGHTS_QUERY,
                )))
            }
            _unsupported_lang => None,
        }
    }

    /// Get language configuration for supported languages
    fn get_language_config(language: &str) -> Result<HighlightLanguageConfig, HighlightError> {
        let lang_id = mcb_domain::ports::validation::LanguageId::from_name(language)
            .ok_or_else(|| HighlightError::UnsupportedLanguage(language.to_owned()))?;

        if let Some(res) = Self::get_language_config_dynamic(lang_id) {
            return res;
        }

        if let Some(res) = Self::get_language_config_static(lang_id) {
            return res;
        }

        Err(HighlightError::UnsupportedLanguage(language.to_owned()))
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

    fn parse_highlight_events(
        highlights: impl Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>>,
    ) -> Result<Vec<HighlightSpan>, HighlightError> {
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

        Ok(spans)
    }

    /// Highlight code using tree-sitter
    fn highlight_code_internal(
        &self,
        code: &str,
        language: &str,
    ) -> Result<HighlightedCode, HighlightError> {
        if code.is_empty() {
            return Ok(HighlightedCode {
                original: code.to_owned(),
                spans: vec![],
                language: language.to_owned(),
            });
        }

        let config_err = |op: &str, e: HighlightError| {
            HighlightError::ConfigurationError(format!("failed to {op} for '{language}': {e}"))
        };

        let lang_config = Self::get_language_config(language)?;

        let config = Self::create_highlight_config(lang_config)
            .map_err(|e| config_err("create highlight config", e))?;

        let mut highlighter = self.highlighter.blocking_lock();

        let highlights = highlighter
            .highlight(&config, code.as_bytes(), None, |_: &str| None)
            .map_err(|e| HighlightError::HighlightingFailed(e.to_string()))?;

        let spans = Self::parse_highlight_events(highlights)?;

        Ok(HighlightedCode {
            original: code.to_owned(),
            spans,
            language: language.to_owned(),
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
        let code = code.to_owned();
        let language = language.to_owned();
        let highlighter = Arc::clone(&self.highlighter);

        let result = tokio::task::spawn_blocking(move || {
            let service = HighlightServiceImpl { highlighter };
            service.highlight_code_internal(&code, &language)
        })
        .await
        .map_err(|e| HighlightError::HighlightingFailed(format!("Blocking task failed: {e}")))?;

        result.map_err(mcb_domain::Error::from)
    }
}

mcb_domain::register_service!(
    mcb_utils::constants::SERVICE_NAME_HIGHLIGHT,
    ServiceBuilder::Highlight(|_context| { Ok(std::sync::Arc::new(HighlightServiceImpl::new())) }),
);
