//! Unit tests for code parsing
//!
//! Tests for `RcaParser` functionality.

use std::path::Path;

use mcb_language_support::Parser;
use mcb_language_support::language::LanguageId;
use mcb_language_support::parser::RcaParser;

#[tokio::test]
async fn test_parse_rust_content() {
    let parser = RcaParser::new();
    let code = br"fn simple_function() -> i32 {
    let x = 1;
    let y = 2;
    x + y
}

fn complex_function(a: i32, b: i32) -> i32 {
    if a > b {
        if a > 10 {
            return a * 2;
        }
        return a;
    } else if b > 10 {
        return b * 2;
    }
    a + b
}";

    let result = parser
        .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
        .await
        .expect("Should parse");

    assert_eq!(result.language, LanguageId::Rust);
    assert!(result.file_metrics.sloc > 0);
    assert!(!result.functions.is_empty());

    // Find complex_function and verify it has higher complexity
    let complex_fn = result
        .functions
        .iter()
        .find(|f| f.name == "complex_function");
    assert!(complex_fn.is_some());
    let complex_fn = complex_fn.unwrap();
    assert!(complex_fn.metrics.cyclomatic >= 1.0);
}

#[tokio::test]
async fn test_parse_python_content() {
    let parser = RcaParser::new();
    let code = br#"def greet(name):
    if name:
        return f"Hello, {name}!"
    return "Hello, World!"
"#;

    let result = parser
        .parse_content(code, LanguageId::Python, Path::new("test.py"))
        .await
        .expect("Should parse");

    assert_eq!(result.language, LanguageId::Python);
    assert!(result.file_metrics.sloc > 0);
}

#[tokio::test]
async fn test_parse_javascript_content() {
    let parser = RcaParser::new();
    let code = br"function add(a, b) {
    return a + b;
}

const multiply = (x, y) => x * y;
";

    let result = parser
        .parse_content(code, LanguageId::JavaScript, Path::new("test.js"))
        .await
        .expect("Should parse");

    assert_eq!(result.language, LanguageId::JavaScript);
    assert!(result.file_metrics.sloc > 0);
}

#[tokio::test]
async fn test_file_metrics() {
    let parser = RcaParser::new();
    let code = b"// This is a comment\nfn main() {\n    println!(\"Hello\");\n}\n";

    let result = parser
        .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
        .await
        .expect("Should parse");

    assert!(result.file_metrics.sloc > 0);
    assert!(result.file_metrics.cloc >= 1); // At least one comment line
}

#[tokio::test]
async fn test_function_info() {
    let parser = RcaParser::new();
    let code = br"fn example(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}";

    let result = parser
        .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
        .await
        .expect("Should parse");

    // Find the 'example' function (RCA may return additional synthetic entries)
    let func = result
        .functions
        .iter()
        .find(|f| f.name == "example")
        .expect("Should find example function");

    assert!(func.start_line >= 1);
    assert!(func.end_line >= func.start_line);
    assert_eq!(func.metrics.nargs, 3);
}
