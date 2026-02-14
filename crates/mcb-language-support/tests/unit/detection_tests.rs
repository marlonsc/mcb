//! Unit tests for language detection
//!
//! Tests for `LanguageDetector` functionality.

use std::path::Path;

use mcb_language_support::detection::LanguageDetector;
use mcb_language_support::language::LanguageId;
use rstest::*;

#[rstest]
#[case("main.rs", LanguageId::Rust)]
#[case("script.py", LanguageId::Python)]
#[case("app.js", LanguageId::JavaScript)]
#[case("component.tsx", LanguageId::TypeScript)]
fn extension_detection(#[case] file_name: &str, #[case] expected: LanguageId) {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.detect(Path::new(file_name), None).unwrap(),
        expected
    );
}

#[rstest]
#[case("main.rs", "rust")]
#[case("script.py", "python")]
fn detect_name(#[case] file_name: &str, #[case] expected: &str) {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.detect_name(Path::new(file_name), None),
        Some(expected.to_string())
    );
}

#[test]
fn content_detection() {
    let detector = LanguageDetector::new();
    let python_content = "#!/usr/bin/env python\nprint('hello')";
    assert_eq!(
        detector
            .detect(Path::new("script.py"), Some(python_content))
            .unwrap(),
        LanguageId::Python
    );
}

#[test]
fn unknown_extension() {
    let detector = LanguageDetector::new();
    assert!(detector.detect(Path::new("file.unknown"), None).is_err());
}

#[test]
fn matches_languages() {
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
fn matches_language_ids() {
    let detector = LanguageDetector::new();

    assert!(detector.matches_language_ids(
        Path::new("main.rs"),
        None,
        &[LanguageId::Rust, LanguageId::Python]
    ));

    assert!(!detector.matches_language_ids(
        Path::new("main.rs"),
        None,
        &[LanguageId::Python, LanguageId::JavaScript]
    ));
}

#[test]
fn supported_languages() {
    let detector = LanguageDetector::new();
    let languages = detector.supported_languages();
    assert!(languages.contains(&LanguageId::Rust));
    assert!(languages.contains(&LanguageId::Python));
    assert_eq!(languages.len(), 7);
}

#[test]
fn supported_language_names() {
    let detector = LanguageDetector::new();
    let names = detector.supported_language_names();
    assert!(names.contains(&"rust".to_string()));
    assert!(names.contains(&"python".to_string()));
    assert_eq!(names.len(), 7);
}
