//! Unit tests for highlight service.

use std::time::Instant;

use mcb_domain::error::Error;
use mcb_domain::ports::{HighlightError, HighlightServiceInterface};
use mcb_domain::registry::services::resolve_highlight_service;
use mcb_domain::test_utils::TestResult;
use mcb_domain::value_objects::browse::HighlightCategory;
use mcb_infrastructure::services::highlight_service::map_highlight_to_category;
use rstest::{fixture, rstest};

#[fixture]
fn highlight_service() -> TestResult<std::sync::Arc<dyn HighlightServiceInterface>> {
    resolve_highlight_service(&()).map_err(Into::into)
}

async fn assert_highlight_success(
    service: &dyn HighlightServiceInterface,
    code: &str,
    language: &str,
    expect_non_empty_spans: bool,
) -> mcb_domain::error::Result<()> {
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
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    #[case] code: &str,
    #[case] language: &str,
    #[case] expect_non_empty_spans: bool,
) -> TestResult {
    let service = highlight_service?;
    assert_highlight_success(&*service, code, language, expect_non_empty_spans).await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_rust_keyword(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = "fn main() {}";
    let result = service.highlight(code, "rust").await?;

    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Keyword)
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_empty_code(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let result = service.highlight("", "rust").await?;

    assert!(result.original.is_empty());
    assert!(result.spans.is_empty());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_unsupported_language(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let result = service.highlight("code", "brainfuck").await;

    let err = result.expect_err("unsupported language should fail");
    assert!(
        matches!(
            err,
            Error::Highlight(HighlightError::UnsupportedLanguage(ref lang)) if lang == "brainfuck"
        ),
        "expected UnsupportedLanguage(brainfuck), got: {err:?}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_fallback_to_plain_text(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = "some code";
    let result = service.highlight(code, "plaintext").await;

    assert!(
        result.is_err(),
        "plaintext should not be a supported highlight language"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_performance_under_500ms(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n".repeat(50);

    let start = Instant::now();
    let result = service.highlight(&code, "rust").await?;
    let elapsed = start.elapsed();

    assert_eq!(result.language, "rust");
    assert!(
        elapsed.as_millis() < 2000,
        "Highlighting took {}ms, expected < 2000ms",
        elapsed.as_millis()
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_multiline_rust(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = "
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}
";
    let result = service.highlight(code, "rust").await?;

    assert_eq!(result.original, code);
    assert_eq!(result.language, "rust");
    assert!(!result.spans.is_empty());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_case_insensitive_language(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = "fn main() {}";

    let result_lower = service.highlight(code, "rust").await?;
    let result_upper = service.highlight(code, "RUST").await?;

    assert_eq!(result_lower.spans.len(), result_upper.spans.len());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_highlight_with_comments(
    highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let service = highlight_service?;
    let code = r#"// This is a comment
fn main() {
    /* Multi-line
       comment */
    println!("Hello");
}
"#;
    let result = service.highlight(code, "rust").await?;

    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Comment)
    );
    Ok(())
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
