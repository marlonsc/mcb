//! Language Detection for Rule Filtering
//!
//! Detects programming language from file extensions and content patterns.
//! Re-exports from mcb-language-support for unified language detection.

// Re-export the shared LanguageDetector for external use
pub use mcb_language_support::LanguageDetector;
// Also expose LanguageId for callers who need it
pub use mcb_language_support::LanguageId;
