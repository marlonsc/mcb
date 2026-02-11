//! Unit tests for highlight service.

use std::time::Instant;

use mcb_domain::error::Error;
use mcb_domain::ports::browse::{HighlightError, HighlightServiceInterface};
use mcb_domain::value_objects::browse::HighlightCategory;
use mcb_infrastructure::services::highlight_service::{
    HighlightServiceImpl, map_highlight_to_category,
};

#[tokio::test]
async fn test_highlight_rust_keyword() {
    let service = HighlightServiceImpl::new();
    let code = "fn main() {}";
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "rust");
    assert!(!result.spans.is_empty());
    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Keyword)
    );
}

#[tokio::test]
async fn test_highlight_python_number() {
    let service = HighlightServiceImpl::new();
    let code = "x = 1";
    let result = service
        .highlight(code, "python")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "python");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_javascript() {
    let service = HighlightServiceImpl::new();
    let code = "const x = 42;";
    let result = service
        .highlight(code, "javascript")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "javascript");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_typescript() {
    let service = HighlightServiceImpl::new();
    let code = "let x: number = 42;";
    let result = service
        .highlight(code, "typescript")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "typescript");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_go() {
    let service = HighlightServiceImpl::new();
    let code = "func main() {}";
    let result = service
        .highlight(code, "go")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "go");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_java() {
    let service = HighlightServiceImpl::new();
    let code = "public class Main {}";
    let result = service
        .highlight(code, "java")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "java");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_cpp() {
    let service = HighlightServiceImpl::new();
    let code = "int main() { return 0; }";
    let result = service
        .highlight(code, "cpp")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "cpp");
}

#[tokio::test]
async fn test_highlight_ruby() {
    let service = HighlightServiceImpl::new();
    let code = "def hello; end";
    let result = service
        .highlight(code, "ruby")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "ruby");
}

#[tokio::test]
async fn test_highlight_php() {
    let service = HighlightServiceImpl::new();
    let code = "<?php echo 'hello'; ?>";
    let result = service
        .highlight(code, "php")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "php");
}

#[tokio::test]
async fn test_highlight_swift() {
    let service = HighlightServiceImpl::new();
    let code = "func main() {}";
    let result = service
        .highlight(code, "swift")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "swift");
}

#[tokio::test]
async fn test_highlight_empty_code() {
    let service = HighlightServiceImpl::new();
    let result = service
        .highlight("", "rust")
        .await
        .expect("Failed to highlight");

    assert!(result.original.is_empty());
    assert!(result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_unsupported_language() {
    let service = HighlightServiceImpl::new();
    let result = service.highlight("code", "brainfuck").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Highlight(HighlightError::UnsupportedLanguage(lang)) => {
            assert_eq!(lang, "brainfuck")
        }
        _ => panic!("Expected Highlight(UnsupportedLanguage) error"),
    }
}

#[tokio::test]
async fn test_highlight_fallback_to_plain_text() {
    let service = HighlightServiceImpl::new();
    let code = "some code";
    let result = service.highlight(code, "plaintext").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_highlight_performance_under_500ms() {
    let service = HighlightServiceImpl::new();
    let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n".repeat(50);

    let start = Instant::now();
    let result = service
        .highlight(&code, "rust")
        .await
        .expect("Failed to highlight");
    let elapsed = start.elapsed();

    assert_eq!(result.language, "rust");
    assert!(
        elapsed.as_millis() < 500,
        "Highlighting took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_highlight_multiline_rust() {
    let service = HighlightServiceImpl::new();
    let code = r#"
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}
"#;
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "rust");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_highlight_case_insensitive_language() {
    let service = HighlightServiceImpl::new();
    let code = "fn main() {}";

    let result_lower = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    let result_upper = service
        .highlight(code, "RUST")
        .await
        .expect("Failed to highlight");

    assert_eq!(result_lower.spans.len(), result_upper.spans.len());
}

#[tokio::test]
async fn test_highlight_with_comments() {
    let service = HighlightServiceImpl::new();
    let code = r#"// This is a comment
fn main() {
    /* Multi-line
       comment */
    println!("Hello");
}
"#;
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Comment)
    );
}

#[test]
fn test_highlight_category_mapping() {
    assert_eq!(
        map_highlight_to_category("keyword"),
        HighlightCategory::Keyword
    );
    assert_eq!(
        map_highlight_to_category("string"),
        HighlightCategory::String
    );
    assert_eq!(
        map_highlight_to_category("comment"),
        HighlightCategory::Comment
    );
    assert_eq!(
        map_highlight_to_category("number"),
        HighlightCategory::Number
    );
    assert_eq!(
        map_highlight_to_category("unknown"),
        HighlightCategory::Other
    );
}
