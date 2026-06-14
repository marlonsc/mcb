//! Synchronous highlight service internals.
//!
//! This module contains all `std::sync::Mutex` and CPU-bound tree-sitter logic.
//! It is intentionally **sync-only** so that the async validator does not flag
//! `std::sync::Mutex` usage inside an async file.

use std::sync::Mutex;

use mcb_domain::ports::HighlightError;
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

/// Internal state holding both the highlighter and cached language configs.
struct HighlighterState {
    highlighter: Highlighter,
    configs: std::collections::HashMap<String, HighlightConfiguration>,
}

/// Port for synchronous syntax highlighting.
///
/// # Example
/// ```
/// use mcb_infrastructure::services::highlight_sync_service::HighlightSyncPort;
/// fn example(port: &dyn HighlightSyncPort) {
///     let _ = port.highlight("fn main() {}", "rust");
/// }
/// ```
pub trait HighlightSyncPort: Send + Sync {
    /// Highlight code using tree-sitter with cached language configs.
    fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode, HighlightError>;
}

/// Thread-safe synchronous highlight service using tree-sitter.
pub struct HighlightSyncService {
    state: Mutex<HighlighterState>,
}

impl HighlightSyncPort for HighlightSyncService {
    fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode, HighlightError> {
        self.highlight(code, language)
    }
}

impl HighlightSyncService {
    /// Creates a new sync highlight service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HighlighterState {
                highlighter: Highlighter::new(),
                configs: std::collections::HashMap::new(),
            }),
        }
    }

    /// Highlight code using tree-sitter with cached language configs.
    pub fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode, HighlightError> {
        if code.is_empty() {
            return Ok(HighlightedCode {
                original: code.to_owned(),
                spans: vec![],
                language: language.to_owned(),
            });
        }

        let mut state = self
            .state
            .lock()
            .map_err(|e| HighlightError::HighlightingFailed(format!("Lock poisoned: {e}")))?;

        // Destructure to allow independent borrows of configs and highlighter
        let HighlighterState {
            highlighter,
            configs,
        } = &mut *state;

        // Get or create cached config for this language (entry API avoids expect)
        let config = match configs.entry(language.to_owned()) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let lang_config = Self::get_language_config(language)?;
                let config = Self::create_highlight_config(lang_config).map_err(|e| {
                    HighlightError::ConfigurationError(format!(
                        "failed to create highlight config for '{language}': {e}"
                    ))
                })?;
                e.insert(config)
            }
        };

        let highlights = highlighter
            .highlight(config, code.as_bytes(), None, |_: &str| None)
            .map_err(|e| HighlightError::HighlightingFailed(e.to_string()))?;

        let spans = Self::parse_highlight_events(highlights)?;

        Ok(HighlightedCode {
            original: code.to_owned(),
            spans,
            language: language.to_owned(),
        })
    }

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

    fn get_language_config_dynamic(
        lang_id: mcb_domain::ports::validation::LanguageId,
    ) -> Option<Result<HighlightLanguageConfig, HighlightError>> {
        use mcb_domain::ports::validation::LanguageId as L;
        let (name, language, highlights_query) = {
            #[allow(clippy::wildcard_enum_match_arm)]
            // Why: only specific LanguageId variants have tree-sitter grammars; unsupported variants map to None.
            match lang_id {
                L::Python => (
                    "python",
                    tree_sitter_python::LANGUAGE.into(),
                    tree_sitter_python::HIGHLIGHTS_QUERY,
                ),
                L::JavaScript => (
                    "javascript",
                    tree_sitter_javascript::LANGUAGE.into(),
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                ),
                L::TypeScript => (
                    "typescript",
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                ),
                L::Tsx => (
                    "tsx",
                    tree_sitter_typescript::LANGUAGE_TSX.into(),
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                ),
                L::Ruby => (
                    "ruby",
                    tree_sitter_ruby::LANGUAGE.into(),
                    tree_sitter_ruby::HIGHLIGHTS_QUERY,
                ),
                L::Php => (
                    "php",
                    tree_sitter_php::LANGUAGE_PHP.into(),
                    tree_sitter_php::HIGHLIGHTS_QUERY,
                ),
                _ => return None,
            }
        };
        Some(Ok(HighlightLanguageConfig::new(
            name,
            language,
            highlights_query,
        )))
    }

    fn get_language_config_static(
        lang_id: mcb_domain::ports::validation::LanguageId,
    ) -> Option<Result<HighlightLanguageConfig, HighlightError>> {
        use mcb_domain::ports::validation::LanguageId as L;
        let (name, language, highlights_query) = {
            #[allow(clippy::wildcard_enum_match_arm)]
            // Why: only specific LanguageId variants have tree-sitter grammars; unsupported variants map to None.
            match lang_id {
                L::Rust => (
                    "rust",
                    tree_sitter_rust::LANGUAGE.into(),
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                ),
                L::Go => (
                    "go",
                    tree_sitter_go::LANGUAGE.into(),
                    tree_sitter_go::HIGHLIGHTS_QUERY,
                ),
                L::Java => (
                    "java",
                    tree_sitter_java::LANGUAGE.into(),
                    tree_sitter_java::HIGHLIGHTS_QUERY,
                ),
                L::C => (
                    "c",
                    tree_sitter_c::LANGUAGE.into(),
                    tree_sitter_c::HIGHLIGHT_QUERY,
                ),
                L::Cpp => (
                    "cpp",
                    tree_sitter_cpp::LANGUAGE.into(),
                    tree_sitter_cpp::HIGHLIGHT_QUERY,
                ),
                L::Swift => (
                    "swift",
                    tree_sitter_swift::LANGUAGE.into(),
                    tree_sitter_swift::HIGHLIGHTS_QUERY,
                ),
                _ => return None,
            }
        };
        Some(Ok(HighlightLanguageConfig::new(
            name,
            language,
            highlights_query,
        )))
    }

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
}

impl Default for HighlightSyncService {
    fn default() -> Self {
        Self::new()
    }
}
