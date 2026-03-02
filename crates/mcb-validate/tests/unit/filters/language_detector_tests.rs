//! Unit tests for language detection (filters).

use rstest::rstest;
use std::path::Path;

use mcb_validate::filters::{LanguageDetector, LanguageId};

#[rstest]
#[case("main.rs", "rust")]
#[case("script.py", "python")]
#[case("app.js", "javascript")]
#[case("component.tsx", "typescript")]
#[case("main.go", "go")]
#[case("script.rb", "ruby")]
#[case("config.yaml", "yaml")]
#[case("Cargo.toml", "toml")]
#[case("data.json", "json")]
#[case("README.md", "markdown")]
#[case("index.html", "html")]
#[case("styles.css", "css")]
#[case("query.sql", "sql")]
#[case("api.proto", "protobuf")]
fn extension_detection(#[case] file: &str, #[case] expected_language: &str) {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.detect_name(Path::new(file), None),
        Some(expected_language.to_owned())
    );
}

#[rstest]
#[case("js", LanguageId::JavaScript)]
#[case("c++", LanguageId::Cpp)]
#[case("sh", LanguageId::Shell)]
#[case("go", LanguageId::Go)]
#[case("yml", LanguageId::Yaml)]
#[case("md", LanguageId::Markdown)]
#[case("proto", LanguageId::Protobuf)]
fn from_name_synonyms(#[case] name: &str, #[case] expected: LanguageId) {
    assert_eq!(LanguageId::from_name(name), Some(expected));
}

#[rstest]
#[case(".go", LanguageId::Go)]
#[case("RB", LanguageId::Ruby)]
#[case("yml", LanguageId::Yaml)]
#[case("TOML", LanguageId::Toml)]
#[case("json", LanguageId::Json)]
#[case("md", LanguageId::Markdown)]
#[case("html", LanguageId::Html)]
#[case("css", LanguageId::Css)]
#[case("sql", LanguageId::Sql)]
#[case("proto", LanguageId::Protobuf)]
fn from_extension_mapping(#[case] ext: &str, #[case] expected: LanguageId) {
    assert_eq!(LanguageId::from_extension(ext), Some(expected));
}

#[rstest]
#[case("Dockerfile", LanguageId::Dockerfile)]
#[case("Makefile", LanguageId::Makefile)]
#[case("GNUmakefile", LanguageId::Makefile)]
#[case("CMakeLists.txt", LanguageId::Cpp)]
fn from_filename_mapping(#[case] filename: &str, #[case] expected: LanguageId) {
    assert_eq!(LanguageId::from_filename(filename), Some(expected));
}

#[rstest]
#[case("#!/usr/bin/env python", LanguageId::Python)]
#[case("#!/bin/bash", LanguageId::Shell)]
#[case("#!/usr/bin/env sh", LanguageId::Shell)]
#[case("#!/usr/bin/env ruby", LanguageId::Ruby)]
fn from_shebang_mapping(#[case] first_line: &str, #[case] expected: LanguageId) {
    assert_eq!(LanguageId::from_shebang(first_line), Some(expected));
}

#[rstest]
#[test]
fn test_content_detection() {
    let detector = LanguageDetector::new();

    let python_content = "#!/usr/bin/env python\nprint('hello')";
    assert_eq!(
        detector.detect_name(Path::new("script.py"), Some(python_content)),
        Some("python".to_owned())
    );
}

#[rstest]
#[test]
fn test_unknown_extension() {
    let detector = LanguageDetector::new();
    assert_eq!(detector.detect_name(Path::new("file.unknown"), None), None);
}

#[rstest]
#[test]
fn test_filename_detection() {
    let detector = LanguageDetector::new();
    assert_eq!(
        detector.detect_name(Path::new("Dockerfile"), None),
        Some("dockerfile".to_owned())
    );
    assert_eq!(
        detector.detect_name(Path::new("Makefile"), None),
        Some("makefile".to_owned())
    );
}

#[rstest]
#[test]
fn test_shebang_detection() {
    let detector = LanguageDetector::new();
    let shell_script = "#!/usr/bin/env bash\necho hello";

    assert_eq!(
        detector.detect_name(Path::new("script.unknown"), Some(shell_script)),
        Some("shell".to_owned())
    );
}

#[rstest]
#[test]
fn test_unknown_existing_file_returns_none() {
    let detector = LanguageDetector::new();

    let path = std::env::temp_dir().join(format!(
        "mcb-validate-language-detector-{}",
        std::process::id()
    ));
    std::fs::write(&path, "not recognizable language content").expect("temp file write");

    assert_eq!(detector.detect_name(&path, None), None);

    std::fs::remove_file(path).expect("temp file cleanup");
}

#[rstest]
#[case("main.rs", vec!["rust".to_owned(), "python".to_owned()], true)]
#[case(
    "main.rs",
    vec!["python".to_owned(), "javascript".to_owned()],
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

#[rstest]
#[test]
fn test_supported_languages() {
    let detector = LanguageDetector::new();
    let languages = detector.supported_language_names();
    assert!(languages.contains(&"rust".to_owned()));
    assert!(languages.contains(&"python".to_owned()));
    assert!(languages.contains(&"javascript".to_owned()));
    assert!(languages.contains(&"typescript".to_owned()));
    assert!(languages.contains(&"go".to_owned()));
    assert!(languages.contains(&"java".to_owned()));
    assert!(languages.contains(&"cpp".to_owned()));
    assert!(languages.contains(&"kotlin".to_owned()));
    assert!(languages.contains(&"ruby".to_owned()));
    assert!(languages.contains(&"shell".to_owned()));
    assert!(languages.contains(&"yaml".to_owned()));
    assert!(languages.contains(&"toml".to_owned()));
    assert!(languages.contains(&"json".to_owned()));
    assert!(languages.contains(&"markdown".to_owned()));
    assert!(languages.contains(&"html".to_owned()));
    assert!(languages.contains(&"css".to_owned()));
    assert!(languages.contains(&"sql".to_owned()));
    assert!(languages.contains(&"dockerfile".to_owned()));
    assert!(languages.contains(&"makefile".to_owned()));
    assert!(languages.contains(&"protobuf".to_owned()));
    assert_eq!(languages.len(), 20);
}
