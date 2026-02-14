//! Unit tests for `FilePatternMatcher`.

use std::path::Path;

use mcb_validate::filters::FilePatternMatcher;
use rstest::rstest;

#[rstest]
#[case("main.rs", true)]
#[case("lib.rs", true)]
#[case("main.py", false)]
#[case("README.md", false)]
#[test]
fn test_simple_includes(#[case] file: &str, #[case] expected: bool) {
    let matcher = FilePatternMatcher::new(&["*.rs".to_string()], &[]).unwrap();
    assert_eq!(matcher.should_include(Path::new(file)), expected);
}

#[test]
fn test_includes_and_excludes() {
    let matcher = FilePatternMatcher::new(
        &["src/**/*.rs".to_string()],
        &["**/test/**".to_string(), "**/*_test.rs".to_string()],
    )
    .unwrap();

    assert!(matcher.should_include(Path::new("src/main.rs")));
    assert!(matcher.should_include(Path::new("src/utils/helper.rs")));
    assert!(!matcher.should_include(Path::new("src/tests/integration_test.rs")));
    assert!(!matcher.should_include(Path::new("src/utils_test.rs")));
    assert!(!matcher.should_include(Path::new("tests/main.rs")));
}

#[test]
fn test_matches_any() {
    let matcher = FilePatternMatcher::default();

    let patterns = vec![
        "src/**/*.rs".to_string(),
        "!**/test/**".to_string(),
        "tests/**/*.py".to_string(),
    ];

    assert!(matcher.matches_any(Path::new("src/main.rs"), &patterns));
    assert!(matcher.matches_any(Path::new("tests/test.py"), &patterns));
    assert!(!matcher.matches_any(Path::new("src/test/main.rs"), &patterns));
    assert!(!matcher.matches_any(Path::new("lib.py"), &patterns));
}

#[test]
fn test_parse_patterns() {
    let patterns = vec![
        "src/**/*.rs".to_string(),
        "!**/test/**".to_string(),
        "tests/**/*.py".to_string(),
        "!**/*.tmp".to_string(),
    ];

    let (includes, excludes) = FilePatternMatcher::parse_patterns(&patterns);

    assert_eq!(includes, vec!["src/**/*.rs", "tests/**/*.py"]);
    assert_eq!(excludes, vec!["**/test/**", "**/*.tmp"]);
}

#[test]
fn test_from_mixed_patterns() {
    let patterns = vec![
        "src/**/*.rs".to_string(),
        "!**/test_utils/**".to_string(),
        "tests/**/*.py".to_string(),
    ];

    let matcher = FilePatternMatcher::from_mixed_patterns(&patterns).unwrap();

    assert!(matcher.should_include(Path::new("src/main.rs")));
    assert!(matcher.should_include(Path::new("tests/test.py")));
    assert!(!matcher.should_include(Path::new("src/test_utils/helpers.rs")));
    assert!(!matcher.should_include(Path::new("lib.py")));
}

#[rstest]
#[case("any/file.rs")]
#[case("any/file.py")]
#[case("any/file.txt")]
#[test]
fn test_default_matcher(#[case] file: &str) {
    let matcher = FilePatternMatcher::default();
    assert!(matcher.should_include(Path::new(file)));
}
