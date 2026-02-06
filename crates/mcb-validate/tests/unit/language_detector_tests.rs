//! Unit tests for language detection (filters).
//!
//! Moved from inline tests in src/filters/language_detector.rs.

use mcb_validate::filters::LanguageDetector;
use std::path::Path;

#[test]
fn test_extension_detection() {
    let detector = LanguageDetector::new();

    assert_eq!(
        detector.detect_name(Path::new("main.rs"), None),
        Some("rust".to_string())
    );
    assert_eq!(
        detector.detect_name(Path::new("script.py"), None),
        Some("python".to_string())
    );
    assert_eq!(
        detector.detect_name(Path::new("app.js"), None),
        Some("javascript".to_string())
    );
    assert_eq!(
        detector.detect_name(Path::new("component.tsx"), None),
        Some("typescript".to_string())
    );
}

#[test]
fn test_content_detection() {
    let detector = LanguageDetector::new();

    let python_content = "#!/usr/bin/env python\nprint('hello')";
    assert_eq!(
        detector.detect_name(Path::new("script.py"), Some(python_content)),
        Some("python".to_string())
    );
}

#[test]
fn test_unknown_extension() {
    let detector = LanguageDetector::new();
    assert_eq!(detector.detect_name(Path::new("file.unknown"), None), None);
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
    let languages = detector.supported_language_names();
    assert!(languages.contains(&"rust".to_string()));
    assert!(languages.contains(&"python".to_string()));
    assert!(languages.len() >= 7);
}
