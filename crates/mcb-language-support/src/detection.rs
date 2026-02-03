//! Language Detection
//!
//! Detects programming language from file extensions and content patterns.
//! Uses Mozilla's rust-code-analysis library for accurate language detection.

use std::path::Path;

use rust_code_analysis::{LANG, guess_language};

use crate::error::{LanguageError, Result};
use crate::language::{LanguageId, LanguageRegistry};

/// Detects programming language from file paths using Mozilla's rust-code-analysis
pub struct LanguageDetector {
    /// Language registry for extension-based lookup
    registry: LanguageRegistry,
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Create a new language detector using Mozilla's rust-code-analysis
    pub fn new() -> Self {
        Self {
            registry: LanguageRegistry::new(),
        }
    }

    /// Detect language from file path
    ///
    /// # Arguments
    /// * `path` - File path to analyze
    /// * `content` - Optional file content for shebang detection
    ///
    /// # Returns
    /// The detected `LanguageId`, or error if unable to detect
    pub fn detect(&self, path: &Path, content: Option<&str>) -> Result<LanguageId> {
        let source = content.map_or_else(
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );

        let rca_result = guess_language(&source, path);

        match rca_result.0 {
            Some(lang) => {
                LanguageId::from_rca_lang(lang).ok_or_else(|| LanguageError::UnsupportedLanguage {
                    language: lang.get_name().to_string(),
                })
            }
            None => Err(LanguageError::DetectionFailed {
                path: path.display().to_string(),
            }),
        }
    }

    /// Try to detect language, returning None instead of error
    pub fn detect_opt(&self, path: &Path, content: Option<&str>) -> Option<LanguageId> {
        self.detect(path, content).ok()
    }

    /// Detect language and return as string name
    ///
    /// # Arguments
    /// * `path` - File path to analyze
    /// * `content` - Optional file content for shebang detection
    ///
    /// # Returns
    /// The detected language name, or None if unable to detect
    pub fn detect_name(&self, path: &Path, content: Option<&str>) -> Option<String> {
        self.detect_opt(path, content)
            .map(|id| id.name().to_string())
    }

    /// Detect language and return RCA LANG enum
    ///
    /// Useful for direct RCA integration.
    pub fn detect_rca_lang(&self, path: &Path, content: Option<&str>) -> Option<LANG> {
        self.detect_opt(path, content).map(|id| id.to_rca_lang())
    }

    /// Check if a file matches any of the specified languages
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `content` - Optional file content
    /// * `allowed_languages` - List of allowed language names
    ///
    /// # Returns
    /// true if the file's language is in the allowed list
    pub fn matches_languages(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed_languages: &[String],
    ) -> bool {
        self.detect_name(path, content)
            .is_some_and(|language| allowed_languages.contains(&language))
    }

    /// Check if a file matches any of the specified language IDs
    pub fn matches_language_ids(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed: &[LanguageId],
    ) -> bool {
        self.detect_opt(path, content)
            .is_some_and(|id| allowed.contains(&id))
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<LanguageId> {
        LanguageId::all().to_vec()
    }

    /// Get all supported language names
    pub fn supported_language_names(&self) -> Vec<String> {
        LanguageId::all()
            .iter()
            .map(|id| id.name().to_string())
            .collect()
    }

    /// Get the underlying registry
    pub fn registry(&self) -> &LanguageRegistry {
        &self.registry
    }
}

// Tests moved to tests/unit/detection_tests.rs
