//! Language Detection for Rule Filtering
//!
//! Detects programming language from file extensions and content patterns.
//! Uses Mozilla's rust-code-analysis library for accurate language detection.

use rust_code_analysis::{LANG, guess_language};
use std::path::Path;

/// Detects programming language from file paths using Mozilla's rust-code-analysis
pub struct LanguageDetector;

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Create a new language detector using Mozilla's rust-code-analysis
    pub fn new() -> Self {
        Self
    }

    /// Detect language from file path
    ///
    /// # Arguments
    /// * `path` - File path to analyze
    /// * `content` - Optional file content for shebang detection
    ///
    /// # Returns
    /// The detected language name, or None if unable to detect
    pub fn detect(&self, path: &Path, content: Option<&str>) -> Option<String> {
        let source = content
            .map(|c| c.as_bytes().to_vec())
            .unwrap_or_else(|| std::fs::read(path).unwrap_or_default());

        guess_language(&source, path).0.map(lang_to_string)
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
        self.detect(path, content)
            .map(|language| allowed_languages.contains(&language))
            .unwrap_or(false)
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        LANG::into_enum_iter().map(lang_to_string).collect()
    }
}

/// Convert LANG enum to string using RCA's get_name()
fn lang_to_string(lang: LANG) -> String {
    lang.get_name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        let detector = LanguageDetector::new();

        assert_eq!(
            detector.detect(Path::new("main.rs"), None),
            Some("rust".to_string())
        );
        assert_eq!(
            detector.detect(Path::new("script.py"), None),
            Some("python".to_string())
        );
        assert_eq!(
            detector.detect(Path::new("app.js"), None),
            Some("javascript".to_string())
        );
        assert_eq!(
            detector.detect(Path::new("component.tsx"), None),
            Some("typescript".to_string())
        );
    }

    #[test]
    fn test_content_detection() {
        let detector = LanguageDetector::new();

        // rust-code-analysis uses extension-based detection primarily
        // Test with proper extension and content
        let python_content = "#!/usr/bin/env python\nprint('hello')";
        assert_eq!(
            detector.detect(Path::new("script.py"), Some(python_content)),
            Some("python".to_string())
        );
    }

    #[test]
    fn test_unknown_extension() {
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect(Path::new("file.unknown"), None), None);
    }

    #[test]
    fn test_matches_languages() {
        let detector = LanguageDetector::new();

        assert!(detector.matches_languages(
            Path::new("main.rs"),
            None,
            &["rust".to_string(), "python".to_string()]
        ));

        assert!(!detector.matches_languages(
            Path::new("main.rs"),
            None,
            &["python".to_string(), "javascript".to_string()]
        ));
    }

    #[test]
    fn test_supported_languages() {
        let detector = LanguageDetector::new();
        let languages = detector.supported_languages();
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
        assert!(languages.len() >= 10);
    }
}
