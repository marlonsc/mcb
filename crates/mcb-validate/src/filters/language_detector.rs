//! Language Detection for Rule Filtering
//!
//! Detects programming language from file extensions and content patterns.
//! Re-exports from mcb-language-support for unified language detection.

use std::path::Path;

// Re-export the shared LanguageDetector for external use
pub use mcb_language_support::LanguageDetector;

// Also expose LanguageId for callers who need it
pub use mcb_language_support::LanguageId;

/// Extension trait providing mcb-validate compatible API (returns `Option<String>`)
///
/// The underlying mcb-language-support `LanguageDetector` has methods that return
/// `Result<LanguageId, Error>` or `Option<LanguageId>`. This trait provides
/// compatibility methods that return `Option<String>` for mcb-validate's existing API.
pub trait LanguageDetectorExt {
    /// Detect language from file path (mcb-validate API)
    ///
    /// # Arguments
    /// * `path` - File path to analyze
    /// * `content` - Optional file content for shebang detection
    ///
    /// # Returns
    /// The detected language name as a string, or None if unable to detect
    fn detect_as_string(&self, path: &Path, content: Option<&str>) -> Option<String>;

    /// Check if a file matches any of the specified languages (by string name)
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `content` - Optional file content
    /// * `allowed_languages` - List of allowed language names
    ///
    /// # Returns
    /// true if the file's language is in the allowed list
    fn matches_languages_by_name(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed_languages: &[String],
    ) -> bool;

    /// Get all supported languages as strings
    fn supported_language_names(&self) -> Vec<String>;
}

impl LanguageDetectorExt for LanguageDetector {
    fn detect_as_string(&self, path: &Path, content: Option<&str>) -> Option<String> {
        // Use the shared detector's detect_name method
        self.detect_name(path, content)
    }

    fn matches_languages_by_name(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed_languages: &[String],
    ) -> bool {
        // Use the shared detector's matches_languages method
        self.matches_languages(path, content, allowed_languages)
    }

    fn supported_language_names(&self) -> Vec<String> {
        // Get supported languages from mcb-language-support
        mcb_language_support::LanguageDetector::supported_language_names(self)
    }
}
