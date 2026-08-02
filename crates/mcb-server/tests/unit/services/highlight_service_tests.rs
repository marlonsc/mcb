//! Syntax highlighting: supported languages, categories, performance.

use std::time::Instant;

use mcb_domain::error::Error;
use mcb_domain::ports::{HighlightError, HighlightServiceInterface};
use mcb_domain::registry::services::resolve_highlight_service;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::value_objects::browse::{HighlightCategory, map_highlight_to_category};
use rstest::{fixture, rstest};

#[fixture]
fn svc() -> TestResult<std::sync::Arc<dyn HighlightServiceInterface>> {
    resolve_highlight_service(&()).map_err(Into::into)
}

// ─── Language support ────────────────────────────────────────────────

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
async fn highlights_supported_languages(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    #[case] code: &str,
    #[case] lang: &str,
    #[case] spans_expected: bool,
) -> TestResult {
    let result = svc?.highlight(code, lang).await?;
    assert_eq!(result.original, code);
    assert_eq!(result.language, lang);
    if spans_expected {
        assert!(!result.spans.is_empty());
    }
    Ok(())
}

#[rstest]
#[tokio::test]
async fn rust_keywords_detected(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let result = svc?.highlight("fn main() {}", "rust").await?;
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
async fn rust_comments_detected(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let code = "// line\nfn main() { /* block */ }";
    let result = svc?.highlight(code, "rust").await?;
    assert!(
        result
            .spans
            .iter()
            .any(|s| s.category == HighlightCategory::Comment)
    );
    Ok(())
}

// ─── Edge cases ──────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn empty_code_returns_empty_spans(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let result = svc?.highlight("", "rust").await?;
    assert!(result.spans.is_empty());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn unsupported_language_rejected(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let err = svc?
        .highlight("code", "brainfuck")
        .await
        .expect_err("must fail");
    assert!(
        matches!(err, Error::Highlight(HighlightError::UnsupportedLanguage(ref l)) if l == "brainfuck")
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn case_insensitive_language_matching(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let s = svc?;
    let lower = s.highlight("fn main() {}", "rust").await?;
    let upper = s.highlight("fn main() {}", "RUST").await?;
    assert_eq!(lower.spans.len(), upper.spans.len());
    Ok(())
}

// ─── Performance ─────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn highlighting_50x_file_completes_under_2s(
    svc: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
) -> TestResult {
    let code = "fn main() {\n    println!(\"Hello\");\n}\n".repeat(50);
    let start = Instant::now();
    svc?.highlight(&code, "rust").await?;
    assert!(
        start.elapsed().as_millis() < 2000,
        "took {}ms",
        start.elapsed().as_millis()
    );
    Ok(())
}

// ─── Category mapping ────────────────────────────────────────────────

#[rstest]
#[case("keyword", HighlightCategory::Keyword)]
#[case("string", HighlightCategory::String)]
#[case("comment", HighlightCategory::Comment)]
#[case("number", HighlightCategory::Number)]
#[case("unknown", HighlightCategory::Other)]
fn token_type_maps_to_category(#[case] token: &str, #[case] expected: HighlightCategory) {
    assert_eq!(map_highlight_to_category(token), expected);
}
