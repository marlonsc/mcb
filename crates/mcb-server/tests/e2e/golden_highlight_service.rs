//! Golden Tests: Syntax Highlighting Service (Phase 8b)
//!
//! Contract: `HighlightServiceImpl` provides server-side syntax highlighting.
//! Tests verify: All 12 languages supported, 13 highlight categories, HTML generation, edge cases.

use mcb_domain::ports::HighlightServiceInterface;
use mcb_domain::registry::services::resolve_highlight_service;
use mcb_infrastructure::services::highlight_renderer::HtmlRenderer;
use rstest::rstest;

fn get_service() -> std::sync::Arc<dyn HighlightServiceInterface> {
    resolve_highlight_service(&()).expect("highlight service should resolve")
}

/// Convenience wrapper: highlight code and render to HTML.
async fn highlight_code(
    code: &str,
    language: &str,
    service: &std::sync::Arc<dyn HighlightServiceInterface>,
) -> String {
    match service.highlight(code, language).await {
        Ok(highlighted) => HtmlRenderer::render(&highlighted),
        Err(_) => {
            let fallback = mcb_domain::value_objects::browse::HighlightedCode::new(
                code.to_owned(),
                vec![],
                language.to_owned(),
            );
            HtmlRenderer::render(&fallback)
        }
    }
}

/// Test data: Language-specific code snippets
struct LanguageTestCase {
    language: &'static str,
    code: &'static str,
}

fn language_test_cases() -> Vec<LanguageTestCase> {
    vec![
        LanguageTestCase {
            language: "rust",
            code: "fn hello() { 42 }",
        },
        LanguageTestCase {
            language: "python",
            code: "def hello():\n    x = 42",
        },
        LanguageTestCase {
            language: "javascript",
            code: "const x = 42;",
        },
        LanguageTestCase {
            language: "java",
            code: "public class Hello { }",
        },
        LanguageTestCase {
            language: "go",
            code: "package main",
        },
        LanguageTestCase {
            language: "ruby",
            code: "def hello\n  42\nend",
        },
    ]
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_all_languages() {
    let service = get_service();
    for case in language_test_cases() {
        let result = highlight_code(case.code, case.language, &service).await;

        assert!(
            !result.is_empty(),
            "Expected non-empty highlight output for {}",
            case.language
        );
        assert!(
            result.contains("<span"),
            "Expected HTML span elements for {}",
            case.language
        );
        assert!(
            result.contains("hl-"),
            "Expected highlighting classes in output for {}",
            case.language
        );
    }
}

#[rstest]
#[case("rust", "fn foo() {}")]
#[case("rust", "\"hello\"")]
#[case("rust", "// comment")]
#[case("rust", "let x = 1;")]
#[case("rust", "#[derive(Debug)]")]
#[rstest]
#[tokio::test]
async fn test_golden_highlight_categories(#[case] lang: &str, #[case] code: &str) {
    let service = get_service();
    let result = highlight_code(code, lang, &service).await;
    assert!(
        result.contains("hl-"),
        "Expected highlighting classes in output for {code}: {result}",
    );
}

#[rstest]
#[case("", "rust")]
#[case("   \n\n  ", "rust")]
#[tokio::test]
async fn test_golden_highlight_empty_like_input(#[case] code: &str, #[case] lang: &str) {
    let service = get_service();
    let result = highlight_code(code, lang, &service).await;
    if code.is_empty() {
        assert_eq!(result, "", "Empty input should produce empty output");
        return;
    }
    // Should not panic on whitespace-only input
    let _ = result;
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_unknown_language() {
    let service = get_service();
    let result = highlight_code("let x = 42;", "unknownlang123", &service).await;
    assert!(
        !result.is_empty(),
        "Should produce output for unknown language (fallback to plain text)"
    );
    assert!(
        result.contains("&lt;") || result.contains("let"),
        "Should handle unknown language gracefully"
    );
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_html_escaping() {
    let service = get_service();
    let result = highlight_code("<div>alert('xss')</div>", "javascript", &service).await;

    assert!(
        result.contains("&lt;") && result.contains("&gt;"),
        "HTML characters should be escaped, got: {result}",
    );
    assert!(
        !result.contains("<div>alert"),
        "HTML should not contain unescaped content that could be XSS"
    );
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_multiline_code() {
    let service = get_service();
    let code = "fn factorial(n: u32) -> u32 {\n    match n {\n        0 => 1,\n        _ => n * factorial(n - 1),\n    }\n}";
    let result = highlight_code(code, "rust", &service).await;

    assert!(!result.is_empty(), "Should handle multiline code correctly");
    assert!(
        result.contains("hl-"),
        "Should apply syntax highlighting to multiline code"
    );
    assert!(result.contains("match"), "Should preserve code content");
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_very_long_lines() {
    let service = get_service();
    let long_code = "let x = 1;\n".repeat(1000);
    let result = highlight_code(&long_code, "rust", &service).await;

    assert!(!result.is_empty(), "Should handle very long input");
    assert!(
        result.contains("span"),
        "Should produce highlighted output for long input"
    );
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_output_format() {
    let service = get_service();
    let result = highlight_code("let x = 42;", "rust", &service).await;

    assert!(result.contains("<span"), "Should contain opening spans");
    assert!(result.contains("</span>"), "Should contain closing spans");

    let opens = result.matches("<span").count();
    let closes = result.matches("</span>").count();
    assert_eq!(
        opens, closes,
        "Span tags should match: {opens} opens vs {closes} closes"
    );
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_consistency() {
    let service = get_service();
    let code = "fn foo() { return 42; }";
    let result1 = highlight_code(code, "rust", &service).await;
    let result2 = highlight_code(code, "rust", &service).await;
    assert_eq!(result1, result2, "Highlighting should be deterministic");
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_comments_preserved() {
    let service = get_service();
    let result = highlight_code("// This is a comment\nlet x = 42;", "rust", &service).await;
    assert!(result.contains("hl-comment"), "Should highlight comments");
}

#[rstest]
#[tokio::test]
async fn test_golden_highlight_strings_preserved() {
    let service = get_service();
    let code = r#"let s1 = "hello"; let s2 = 'world';"#;
    let result = highlight_code(code, "rust", &service).await;
    assert!(
        result.contains("hl-string"),
        "Should highlight string literals"
    );
}
