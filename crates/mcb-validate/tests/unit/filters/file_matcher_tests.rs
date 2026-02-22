//! Unit tests for `FilePatternMatcher`.

use rstest::rstest;
use std::path::Path;

use mcb_validate::filters::FilePatternMatcher;

#[rstest]
#[case("main.rs", true)]
#[case("lib.rs", true)]
#[case("main.py", false)]
#[case("README.md", false)]
fn simple_includes(#[case] file: &str, #[case] expected: bool) {
    let matcher = FilePatternMatcher::new(&["*.rs".to_owned()], &[]).unwrap();
    assert_eq!(matcher.should_include(Path::new(file)), expected);
}

#[rstest]
#[case("src/main.rs", true)]
#[case("src/utils/helper.rs", true)]
#[case("src/tests/integration_test.rs", false)]
#[case("src/utils_test.rs", false)]
#[case("tests/main.rs", false)]
fn includes_and_excludes(#[case] file: &str, #[case] expected: bool) {
    let matcher = FilePatternMatcher::new(
        &["src/**/*.rs".to_owned()],
        &["**/test/**".to_owned(), "**/*_test.rs".to_owned()],
    )
    .unwrap();

    assert_eq!(matcher.should_include(Path::new(file)), expected);
}

#[rstest]
#[case("src/main.rs", true)]
#[case("tests/test.py", true)]
#[case("src/test/main.rs", false)]
#[case("lib.py", false)]
fn matches_any(#[case] file: &str, #[case] expected: bool) {
    let matcher = FilePatternMatcher::default();

    let patterns = vec![
        "src/**/*.rs".to_owned(),
        "!**/test/**".to_owned(),
        "tests/**/*.py".to_owned(),
    ];

    assert_eq!(matcher.matches_any(Path::new(file), &patterns), expected);
}

#[rstest]
fn parse_patterns() {
    let patterns = vec![
        "src/**/*.rs".to_owned(),
        "!**/test/**".to_owned(),
        "tests/**/*.py".to_owned(),
        "!**/*.tmp".to_owned(),
    ];

    let (includes, excludes) = FilePatternMatcher::parse_patterns(&patterns);

    assert_eq!(includes, vec!["src/**/*.rs", "tests/**/*.py"]);
    assert_eq!(excludes, vec!["**/test/**", "**/*.tmp"]);
}

#[rstest]
#[case("src/main.rs", true)]
#[case("tests/test.py", true)]
#[case("src/utils/helpers.rs", false)]
#[case("lib.py", false)]
fn from_mixed_patterns(#[case] file: &str, #[case] expected: bool) {
    let patterns = vec![
        "src/**/*.rs".to_owned(),
        "!**/utils/**".to_owned(),
        "tests/**/*.py".to_owned(),
    ];

    let matcher = FilePatternMatcher::from_mixed_patterns(&patterns).unwrap();

    assert_eq!(matcher.should_include(Path::new(file)), expected);
}

#[rstest]
#[case("any/file.rs")]
#[case("any/file.py")]
#[case("any/file.txt")]
fn default_matcher(#[case] file: &str) {
    let matcher = FilePatternMatcher::default();
    assert!(matcher.should_include(Path::new(file)));
}
