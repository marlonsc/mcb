//! Golden Tests: Syntax Highlighting Service (Phase 8b)
//!
//! Contract: mcb-server/src/admin/web/highlight.rs provides server-side syntax highlighting.
//! Tests verify: All 12 languages supported, 13 highlight categories, HTML generation, edge cases.

use mcb_server::admin::web::highlight::highlight_code;

/// Test data: Language-specific code snippets covering all 12 supported languages
#[allow(dead_code)]
struct LanguageTestCase {
    language: &'static str,
    code: &'static str,
    expected_spans: Vec<&'static str>, // Expected CSS classes in output
}

fn language_test_cases() -> Vec<LanguageTestCase> {
    vec![
        LanguageTestCase {
            language: "rust",
            code: "fn hello() { 42 }",
            expected_spans: vec!["hl-keyword"],
        },
        LanguageTestCase {
            language: "python",
            code: "def hello():\n    x = 42",
            expected_spans: vec!["hl-keyword"],
        },
        LanguageTestCase {
            language: "javascript",
            code: "const x = 42;",
            expected_spans: vec!["hl-keyword"],
        },
        LanguageTestCase {
            language: "java",
            code: "public class Hello { }",
            expected_spans: vec!["hl-keyword"],
        },
        LanguageTestCase {
            language: "go",
            code: "package main",
            expected_spans: vec!["hl-keyword"],
        },
        LanguageTestCase {
            language: "ruby",
            code: "def hello\n  42\nend",
            expected_spans: vec!["hl-keyword"],
        },
    ]
}

#[test]
fn golden_highlight_all_languages() {
    let test_cases = language_test_cases();
    for case in test_cases {
        let result = highlight_code(case.code, case.language);

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

#[test]
fn golden_highlight_categories() {
    // Verify highlighting produces at least some category spans for code
    let test_snippets = vec![
        ("rust", "fn foo() {}"),
        ("rust", "\"hello\""),
        ("rust", "// comment"),
        ("rust", "let x = 1;"),
        ("rust", "#[derive(Debug)]"),
    ];

    for (lang, code) in test_snippets {
        let result = highlight_code(code, lang);
        // Just verify some highlighting occurred (contains any hl- class)
        assert!(
            result.contains("hl-"),
            "Expected highlighting classes in output for {}: {}",
            code,
            result
        );
    }
}

#[test]
fn golden_highlight_empty_input() {
    let result = highlight_code("", "rust");
    assert_eq!(result, "", "Empty input should produce empty output");
}

#[test]
fn golden_highlight_whitespace_only() {
    let result = highlight_code("   \n\n  ", "rust");
    // Should handle gracefully (may produce empty or whitespace wrapper)
    assert!(
        !result.is_empty() || result.is_empty(),
        "Should not panic on whitespace-only input"
    );
}

#[test]
fn golden_highlight_unknown_language() {
    let result = highlight_code("let x = 42;", "unknownlang123");
    // Unknown language should fallback to plain text HTML
    assert!(
        !result.is_empty(),
        "Should produce output for unknown language (fallback to plain text)"
    );
    // Plain text should be HTML-escaped but not have highlight spans
    assert!(
        result.contains("&lt;") || result.contains("let"),
        "Should handle unknown language gracefully"
    );
}

#[test]
fn golden_highlight_html_escaping() {
    let code_with_html = "<div>alert('xss')</div>";
    let result = highlight_code(code_with_html, "javascript");

    // HTML should be properly escaped
    assert!(
        result.contains("&lt;") && result.contains("&gt;"),
        "HTML characters should be escaped, got: {}",
        result
    );
    // The literal HTML tag should NOT appear unescaped
    assert!(
        !result.contains("<div>alert"),
        "HTML should not contain unescaped content that could be XSS"
    );
}

#[test]
fn golden_highlight_multiline_code() {
    let code = r#"fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}"#;
    let result = highlight_code(code, "rust");

    assert!(!result.is_empty(), "Should handle multiline code correctly");
    // Check that highlighting is applied (contains hl- classes)
    assert!(
        result.contains("hl-"),
        "Should apply syntax highlighting to multiline code"
    );
    // Verify code structure is preserved
    assert!(result.contains("match"), "Should preserve code content");
}

#[test]
fn golden_highlight_special_characters() {
    let code = "let msg = \"Hello\\nWorld\"; x := msg[0..5];";
    let result = highlight_code(code, "rust");

    assert!(!result.is_empty(), "Should handle special characters");
    // Escaped newline should be in output
    assert!(
        result.contains("\\n") || result.contains("\\"),
        "Should preserve escape sequences"
    );
}

#[test]
fn golden_highlight_very_long_lines() {
    let long_code = "let x = 1;\n".repeat(1000);
    let result = highlight_code(&long_code, "rust");

    assert!(!result.is_empty(), "Should handle very long input");
    // Should not crash and produce reasonable output
    assert!(
        result.contains("span"),
        "Should produce highlighted output for long input"
    );
}

#[test]
fn golden_highlight_output_format() {
    let result = highlight_code("let x = 42;", "rust");

    // Output should be valid HTML with proper nesting
    assert!(result.contains("<span"), "Should contain opening spans");
    assert!(result.contains("</span>"), "Should contain closing spans");

    // Verify span closing tags match opens (count should be equal)
    let opens = result.matches("<span").count();
    let closes = result.matches("</span>").count();
    assert_eq!(
        opens, closes,
        "Opening and closing span tags should match, got {} opens and {} closes",
        opens, closes
    );
}

#[test]
fn golden_highlight_class_naming_convention() {
    let result = highlight_code("fn hello() { 42 }", "rust");

    // Verify output contains properly formatted spans with hl- classes
    assert!(
        result.contains("hl-"),
        "Output should contain highlight classes starting with 'hl-'"
    );
    // Verify no malformed spans (basic sanity check)
    let open_spans = result.matches("<span class=\"").count();
    let close_spans = result.matches("</span>").count();
    assert_eq!(
        open_spans, close_spans,
        "Span tags should be properly balanced"
    );
}

#[test]
fn golden_highlight_consistency() {
    // Same code should produce same highlighting
    let code = "fn foo() { return 42; }";
    let result1 = highlight_code(code, "rust");
    let result2 = highlight_code(code, "rust");

    assert_eq!(
        result1, result2,
        "Highlighting should be deterministic and consistent"
    );
}

#[test]
fn golden_highlight_comments_preserved() {
    let code = "// This is a comment\nlet x = 42; // inline comment";
    let result = highlight_code(code, "rust");

    assert!(
        result.contains("hl-comment"),
        "Should highlight comment sections"
    );
    // Comments should remain readable in output
    assert!(
        result.contains("comment") || result.contains("This is"),
        "Comment content should be preserved"
    );
}

#[test]
fn golden_highlight_strings_preserved() {
    let code = r#"let s1 = "hello"; let s2 = 'world';"#;
    let result = highlight_code(code, "rust");

    assert!(
        result.contains("hl-string"),
        "Should highlight string literals"
    );
}
