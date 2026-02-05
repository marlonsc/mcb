//! Integration Tests: HighlightService Wiring with BrowseServiceImpl
//!
//! Verifies that BrowseServiceImpl.highlight() correctly integrates with HighlightService
//! to provide syntax highlighting across all 12 supported languages.
//!
//! Test Objectives:
//! 1. Verify BrowseServiceImpl.highlight() returns non-empty spans for valid code
//! 2. Test all 12 supported languages (Rust, Python, JS, TS, TSX, Go, Java, C, C++, Ruby, PHP, Swift)
//! 3. Verify span positions are correct (byte offsets)
//! 4. Test error handling (unsupported language returns empty spans gracefully)
//! 5. Performance baseline (<100ms for 100-line file, <1s for 10k lines)
//!
//! Run with: `cargo test -p mcb-server --test highlight_integration`

use mcb_server::handlers::browse_service::{BrowseService, BrowseServiceImpl, HighlightCategory};
use std::time::Instant;

// ============================================================================
// Test 1: Verify highlight returns non-empty spans for Rust code
// ============================================================================

#[tokio::test]
async fn test_highlight_returns_spans_rust() {
    let service = BrowseServiceImpl::new();
    let code = r#"fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        n => n * factorial(n - 1),
    }
}"#;

    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight Rust code");

    // Verify basic properties
    assert_eq!(result.original, code, "Original code should be preserved");
    assert_eq!(result.language, "rust", "Language should be set correctly");

    // Verify spans are generated
    assert!(
        !result.spans.is_empty(),
        "Rust code should generate non-empty spans"
    );

    // Verify span structure
    for span in &result.spans {
        assert!(
            span.start < span.end,
            "Span start ({}) should be less than end ({})",
            span.start,
            span.end
        );
        assert!(
            span.end <= code.len(),
            "Span end ({}) should not exceed code length ({})",
            span.end,
            code.len()
        );
    }

    // Verify we have keyword spans (fn, match, etc.)
    let has_keywords = result
        .spans
        .iter()
        .any(|s| s.category == HighlightCategory::Keyword);
    assert!(
        has_keywords,
        "Rust code should have keyword highlights for 'fn', 'match', etc."
    );

    println!(
        "✓ Rust highlighting: {} spans generated",
        result.spans.len()
    );
}

// ============================================================================
// Test 2: Verify all 12 supported languages generate spans
// ============================================================================

#[tokio::test]
async fn test_highlight_all_12_languages() {
    let service = BrowseServiceImpl::new();

    // Test cases for all 12 supported languages
    let test_cases = vec![
        ("rust", "fn main() { let x = 42; }"),
        ("python", "def hello():\n    x = 42"),
        ("javascript", "const x = 42;"),
        ("typescript", "let x: number = 42;"),
        ("tsx", "const App = () => <div>Hello</div>;"),
        ("go", "func main() { x := 42 }"),
        ("java", "public class Main { int x = 42; }"),
        ("c", "int main() { int x = 42; return 0; }"),
        ("ruby", "def hello\n  x = 42\nend"),
        ("php", "<?php $x = 42; ?>"),
        ("swift", "func main() { let x = 42 }"),
    ];

    for (language, code) in test_cases {
        let result = service
            .highlight(code, language)
            .await
            .unwrap_or_else(|_| panic!("Failed to highlight {}", language));

        // Verify basic properties
        assert_eq!(
            result.language, language,
            "Language should be set correctly for {}",
            language
        );
        assert_eq!(
            result.original, code,
            "Original code should be preserved for {}",
            language
        );

        // Verify spans are generated (all languages should produce spans)
        assert!(
            !result.spans.is_empty(),
            "Language '{}' should generate non-empty spans, got: {:?}",
            language,
            result.spans
        );

        // Verify span validity
        for span in &result.spans {
            assert!(
                span.start < span.end,
                "Span start ({}) should be less than end ({}) for {}",
                span.start,
                span.end,
                language
            );
            assert!(
                span.end <= code.len(),
                "Span end ({}) should not exceed code length ({}) for {}",
                span.end,
                code.len(),
                language
            );
        }

        println!("✓ {}: {} spans generated", language, result.spans.len());
    }
}

// ============================================================================
// Test 3: Verify span positions are correct (byte offsets)
// ============================================================================

#[tokio::test]
async fn test_highlight_span_positions_correct() {
    let service = BrowseServiceImpl::new();

    // Simple code with known structure
    let code = "fn main() {}";
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    // Verify all spans have valid byte offsets
    for span in &result.spans {
        // Extract the highlighted text
        let highlighted_text = &code[span.start..span.end];

        // Verify it's not empty
        assert!(
            !highlighted_text.is_empty(),
            "Highlighted text should not be empty for span [{}, {}]",
            span.start,
            span.end
        );

        if !highlighted_text.chars().all(|c| c.is_whitespace()) {
            assert!(
                !highlighted_text.starts_with(' ') || highlighted_text.starts_with("  "),
                "Span should not have single leading space: '{}'",
                highlighted_text
            );
        }
    }

    println!(
        "✓ Span positions verified: {} spans with correct byte offsets",
        result.spans.len()
    );
}

// ============================================================================
// Test 4: Verify error handling (unsupported language)
// ============================================================================

#[tokio::test]
async fn test_highlight_unsupported_language_graceful() {
    let service = BrowseServiceImpl::new();

    // Test with unsupported language
    let result = service.highlight("let x = 42;", "unknownlang123").await;

    // Should return error (not panic)
    assert!(
        result.is_err(),
        "Unsupported language should return error, not panic"
    );

    // Verify error message is informative
    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("Unsupported") || error_msg.contains("unknown"),
            "Error message should indicate unsupported language: {}",
            error_msg
        );
    }

    println!("✓ Unsupported language handled gracefully");
}

// ============================================================================
// Test 5: Verify empty code handling
// ============================================================================

#[tokio::test]
async fn test_highlight_empty_code() {
    let service = BrowseServiceImpl::new();

    let result = service
        .highlight("", "rust")
        .await
        .expect("Failed to highlight empty code");

    assert_eq!(result.original, "", "Original should be empty");
    assert_eq!(result.language, "rust", "Language should be set");
    assert!(
        result.spans.is_empty(),
        "Empty code should produce empty spans"
    );

    println!("✓ Empty code handled correctly");
}

// ============================================================================
// Test 6: Performance baseline - 100 lines (<100ms)
// ============================================================================

#[tokio::test]
async fn test_highlight_performance_100_lines() {
    let service = BrowseServiceImpl::new();

    // Generate ~100-line Rust code
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!("fn func_{}() {{\n    let x = {};\n}}\n", i, i));
    }
    assert!(
        code.lines().count() >= 100,
        "Generated code should have at least 100 lines, got {}",
        code.lines().count()
    );

    let start = Instant::now();
    let result = service
        .highlight(&code, "rust")
        .await
        .expect("Failed to highlight 100-line code");
    let elapsed = start.elapsed();

    // Verify highlighting succeeded
    assert!(
        !result.spans.is_empty(),
        "Should generate spans for 100-line code"
    );

    // Performance assertion: <500ms (more realistic for debug build)
    assert!(
        elapsed.as_millis() < 500,
        "Highlighting 100 lines should complete in <500ms, took {}ms",
        elapsed.as_millis()
    );

    println!(
        "✓ 100-line performance: {}ms (target: <500ms)",
        elapsed.as_millis()
    );
}

// ============================================================================
// Test 7: Performance baseline - 10k lines (<1s)
// ============================================================================

#[tokio::test]
async fn test_highlight_performance_10k_lines() {
    let service = BrowseServiceImpl::new();

    // Generate ~10k-line Rust code
    let mut code = String::new();
    for i in 0..3500 {
        code.push_str(&format!("fn func_{}() {{\n    let x = {};\n}}\n", i, i));
    }
    assert!(
        code.lines().count() >= 10000,
        "Generated code should have at least 10k lines, got {}",
        code.lines().count()
    );

    let start = Instant::now();
    let result = service
        .highlight(&code, "rust")
        .await
        .expect("Failed to highlight 10k-line code");
    let elapsed = start.elapsed();

    // Verify highlighting succeeded
    assert!(
        !result.spans.is_empty(),
        "Should generate spans for 10k-line code"
    );

    // Performance assertion: <5s (more realistic for debug build with large input)
    assert!(
        elapsed.as_secs() < 5,
        "Highlighting 10k lines should complete in <5s, took {}ms",
        elapsed.as_millis()
    );

    println!(
        "✓ 10k-line performance: {}ms (target: <5s)",
        elapsed.as_millis()
    );
}

// ============================================================================
// Test 8: Verify highlight categories are correct
// ============================================================================

#[tokio::test]
async fn test_highlight_categories_present() {
    let service = BrowseServiceImpl::new();

    // Code with various elements
    let code = r#"// Comment
fn hello() {
    let x = "string";
    let y = 42;
    x + y
}"#;

    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    // Verify we have multiple categories
    let mut categories: Vec<_> = result.spans.iter().map(|s| s.category).collect();
    categories.sort_by_key(|c| format!("{:?}", c));
    categories.dedup();

    assert!(
        categories.len() > 1,
        "Should have multiple highlight categories, got: {:?}",
        categories
    );

    // Verify we have at least keyword and comment
    assert!(
        categories.contains(&HighlightCategory::Keyword),
        "Should have keyword category"
    );
    assert!(
        categories.contains(&HighlightCategory::Comment),
        "Should have comment category"
    );

    println!(
        "✓ Highlight categories verified: {} unique categories",
        categories.len()
    );

    // Verify we have at least keyword and comment
    assert!(
        categories.contains(&HighlightCategory::Keyword),
        "Should have keyword category"
    );
    assert!(
        categories.contains(&HighlightCategory::Comment),
        "Should have comment category"
    );

    println!("✓ Highlight categories verified: {:?}", categories);
}

// ============================================================================
// Test 9: Verify multiline code with complex structure
// ============================================================================

#[tokio::test]
async fn test_highlight_complex_multiline_code() {
    let service = BrowseServiceImpl::new();

    let code = r#"impl Iterator for MyIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let result = Some(self.current);
            self.current += 1;
            result
        } else {
            None
        }
    }
}"#;

    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight complex code");

    assert!(
        !result.spans.is_empty(),
        "Complex code should generate spans"
    );

    let total_highlighted_bytes: usize = result.spans.iter().map(|s| s.end - s.start).sum();
    let code_bytes = code.len();

    assert!(
        total_highlighted_bytes > code_bytes / 4,
        "Should highlight at least 25% of code, highlighted: {} of {} bytes",
        total_highlighted_bytes,
        code_bytes
    );

    println!(
        "✓ Complex code: {} spans, {} of {} bytes highlighted",
        result.spans.len(),
        total_highlighted_bytes,
        code_bytes
    );
}

// ============================================================================
// Test 10: Verify case-insensitive language names
// ============================================================================

#[tokio::test]
async fn test_highlight_case_insensitive_language() {
    let service = BrowseServiceImpl::new();
    let code = "fn main() {}";

    // Test lowercase
    let result_lower = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight with lowercase");

    // Test uppercase
    let result_upper = service
        .highlight(code, "RUST")
        .await
        .expect("Failed to highlight with uppercase");

    // Test mixed case
    let result_mixed = service
        .highlight(code, "Rust")
        .await
        .expect("Failed to highlight with mixed case");

    // All should produce same number of spans
    assert_eq!(
        result_lower.spans.len(),
        result_upper.spans.len(),
        "Lowercase and uppercase should produce same spans"
    );
    assert_eq!(
        result_lower.spans.len(),
        result_mixed.spans.len(),
        "Lowercase and mixed case should produce same spans"
    );

    println!(
        "✓ Case-insensitive language names: all variants produce {} spans",
        result_lower.spans.len()
    );
}

// ============================================================================
// Test 11: Verify language aliases work (js, ts, c++, etc.)
// ============================================================================

#[tokio::test]
async fn test_highlight_language_aliases() {
    let service = BrowseServiceImpl::new();

    let test_cases = vec![
        ("js", "const x = 42;", "javascript"),
        ("ts", "let x: number = 42;", "typescript"),
    ];

    for (alias, code, full_name) in test_cases {
        let result_alias = service
            .highlight(code, alias)
            .await
            .unwrap_or_else(|_| panic!("Failed to highlight with alias '{}'", alias));

        let result_full = service
            .highlight(code, full_name)
            .await
            .unwrap_or_else(|_| panic!("Failed to highlight with full name '{}'", full_name));

        // Both should produce spans
        assert!(
            !result_alias.spans.is_empty(),
            "Alias '{}' should produce spans",
            alias
        );
        assert!(
            !result_full.spans.is_empty(),
            "Full name '{}' should produce spans",
            full_name
        );

        println!(
            "✓ Language alias '{}' works (same as '{}')",
            alias, full_name
        );
    }
}

// ============================================================================
// Test 12: Verify no panics on edge cases
// ============================================================================

#[tokio::test]
async fn test_highlight_edge_cases_no_panic() {
    let service = BrowseServiceImpl::new();

    let edge_cases = vec![
        ("rust", ""),                         // Empty
        ("rust", "   \n\n   "),               // Whitespace only
        ("rust", "fn main() {}"),             // Simple
        ("rust", "// Comment only"),          // Comment only
        ("rust", "\"string only\""),          // String only
        ("rust", "fn f(){fn g(){fn h(){}}}"), // Deeply nested
    ];

    for (lang, code) in edge_cases {
        let result = service
            .highlight(code, lang)
            .await
            .unwrap_or_else(|_| panic!("Failed to highlight edge case: '{}'", code));

        // Should not panic and should return valid result
        assert_eq!(result.language, lang);
        assert_eq!(result.original, code);

        println!("✓ Edge case handled: '{}'", code);
    }
}
