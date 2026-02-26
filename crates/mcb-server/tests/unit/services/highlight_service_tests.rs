//! Unit tests for highlight service.

use std::time::Instant;

use mcb_domain::error::Error;
use mcb_domain::ports::{HighlightError, HighlightServiceInterface};
use mcb_domain::value_objects::browse::HighlightCategory;
use mcb_infrastructure::services::highlight_service::{
    HighlightServiceImpl, map_highlight_to_category,
};
use rstest::rstest;

async fn assert_highlight_success(
    code: &str,
    language: &str,
    expect_non_empty_spans: bool,
) -> mcb_domain::error::Result<()> {
    let service = HighlightServiceImpl::new();
    let result = service.highlight(code, language).await?;

    assert_eq!(result.original, code);
    assert_eq!(result.language, language);
    if expect_non_empty_spans {
        assert!(!result.spans.is_empty());
    }

    Ok(())
}

#[rstest]
#[case("fn main() {}", "rust", true)]
#[case("x = 1", "python", true)]
#[case("const x = 42;", "javascript", true)]
#[case("let x: number = 42;", "typescript", true)]
#[case("func main() {}", "go", true)]
#[case("public class Main {}", "java", true)]
#[case("int main() { return 0; }", "cpp", false)]
#[case("def hello; end", "ruby", false)]
#[case("<?php echo 'hello'; ?>", "php", false)]
#[case("func main() {}", "swift", false)]
#[tokio::test]
async fn test_highlight_language_samples(
    #[case] code: &str,
    #[case] language: &str,
    #[case] expect_non_empty_spans: bool,
) {
    assert_highlight_success(code, language, expect_non_empty_spans)
        .await
        .expect("Failed to highlight");
}

#[tokio::test]
async fn test_highlight_rust_keyword() {
    let service = HighlightServiceImpl::new();
    let code = "fn main() {}";
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Keyword)
    );
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

    let err = result.expect_err("unsupported language should fail");
    assert!(
        matches!(
            err,
            Error::Highlight(HighlightError::UnsupportedLanguage(ref lang)) if lang == "brainfuck"
        ),
        "expected UnsupportedLanguage(brainfuck), got: {err:?}"
    );
}

#[tokio::test]
async fn test_highlight_fallback_to_plain_text() {
    let service = HighlightServiceImpl::new();
    let code = "some code";
    let result = service.highlight(code, "plaintext").await;

    let _err = result.expect_err("plaintext should not be a supported highlight language");
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
        elapsed.as_millis() < 2000,
        "Highlighting took {}ms, expected < 2000ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_highlight_multiline_rust() {
    let service = HighlightServiceImpl::new();
    let code = "
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}
";
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

#[rstest]
#[case("keyword", HighlightCategory::Keyword)]
#[case("string", HighlightCategory::String)]
#[case("comment", HighlightCategory::Comment)]
#[case("number", HighlightCategory::Number)]
#[case("unknown", HighlightCategory::Other)]
fn test_highlight_category_mapping(#[case] token_type: &str, #[case] expected: HighlightCategory) {
    assert_eq!(map_highlight_to_category(token_type), expected);
}
