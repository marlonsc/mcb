//! Unit tests for language detection (filters).

use std::path::Path;

use mcb_validate::filters::LanguageDetector;
use rstest::*;

#[rstest]
#[case("main.rs", "rust")]
#[case("script.py", "python")]
#[case("app.js", "javascript")]
#[case("component.tsx", "typescript")]
fn extension_detection(#[case] file: &str, #[case] expected_language: &str) {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.detect_name(Path::new(file), None),
        Some(expected_language.to_string())
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

#[rstest]
#[case("main.rs", vec!["rust".to_string(), "python".to_string()], true)]
#[case(
    "main.rs",
    vec!["python".to_string(), "javascript".to_string()],
    false
)]
fn matches_languages(
    #[case] file: &str,
    #[case] allowed_languages: Vec<String>,
    #[case] expected: bool,
) {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.matches_languages(Path::new(file), None, &allowed_languages),
        expected
    );
}

#[test]
fn test_supported_languages() {
    let detector = LanguageDetector::new();
    let languages = detector.supported_language_names();
    assert!(languages.contains(&"rust".to_string()));
    assert!(languages.contains(&"python".to_string()));
    assert!(languages.len() >= 7);
}
