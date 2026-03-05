//! Unit tests for `mcb_domain::utils::analysis` utilities.
//!
//! Tests cover: complexity scoring, function body extraction, brace counting,
//! exempt symbols, symbol occurrence counting, function collection,
//! dead code detection, and TDG scoring.

use std::path::PathBuf;

use mcb_domain::ports::providers::analysis::AnalysisFinding;
use mcb_domain::utils::analysis::{
    FunctionRecord, collect_functions, compute_complexity_score, compute_tdg_scores,
    count_balanced_block_lines, count_symbol_occurrences, detect_dead_functions,
    extract_function_body, filter_complex_functions, is_exempt_symbol,
};
use rstest::rstest;

// ---------------------------------------------------------------------------
// compute_complexity_score
// ---------------------------------------------------------------------------

#[rstest]
#[case("fn foo() { }", 0, 1)] // empty body = base 1
#[case("fn foo() { if true {} }", 0, 2)] // one if = 2
#[case("fn foo() { if a {} else if b {} }", 0, 3)] // two ifs = 3
#[case("fn foo() { for x in items { if a {} } }", 0, 3)] // for + if = 3
#[case("fn foo() { while true { loop {} } }", 0, 3)] // while + loop = 3
#[case("fn foo() { match x { _ => () } }", 0, 2)] // match = 2
#[case("fn foo() { if a && b || c {} }", 0, 4)] // if + && + || = 4
fn complexity_score_branching(#[case] code: &str, #[case] pos: usize, #[case] expected: u32) {
    let score = compute_complexity_score(code, pos).expect("should not fail");
    assert_eq!(score, expected, "complexity for: {code}");
}

#[rstest]
fn complexity_score_no_brace_returns_base() {
    // When there's no opening brace, extract_function_body returns None → default body
    let score = compute_complexity_score("fn foo()", 0).expect("should not fail");
    assert_eq!(score, 1);
}

// ---------------------------------------------------------------------------
// extract_function_body
// ---------------------------------------------------------------------------

#[rstest]
fn extract_body_simple() {
    let code = "fn foo() { let x = 1; }";
    let body = extract_function_body(code, 0);
    assert_eq!(body.as_deref(), Some("{ let x = 1; }"));
}

#[rstest]
fn extract_body_nested_braces() {
    let code = "fn bar() { if true { inner() } }";
    let body = extract_function_body(code, 0);
    assert_eq!(body.as_deref(), Some("{ if true { inner() } }"));
}

#[rstest]
fn extract_body_no_opening_brace() {
    let code = "fn baz()";
    assert!(extract_function_body(code, 0).is_none());
}

#[rstest]
fn extract_body_unbalanced() {
    let code = "fn qux() { let x = 1;";
    assert!(extract_function_body(code, 0).is_none());
}

#[rstest]
fn extract_body_offset() {
    let code = "other code\nfn foo() { body }";
    let body = extract_function_body(code, 11); // starts at "fn foo"
    assert_eq!(body.as_deref(), Some("{ body }"));
}

// ---------------------------------------------------------------------------
// count_balanced_block_lines
// ---------------------------------------------------------------------------

#[rstest]
fn balanced_block_single_line() {
    let lines = vec!["fn foo() { }"];
    assert_eq!(count_balanced_block_lines(lines.into_iter(), 10), Some(1));
}

#[rstest]
fn balanced_block_multi_line() {
    let lines = vec!["fn foo() {", "    let x = 1;", "}"];
    assert_eq!(count_balanced_block_lines(lines.into_iter(), 10), Some(3));
}

#[rstest]
fn balanced_block_nested() {
    let lines = vec!["fn foo() {", "  if true {", "    x()", "  }", "}"];
    assert_eq!(count_balanced_block_lines(lines.into_iter(), 10), Some(5));
}

#[rstest]
fn balanced_block_no_brace_within_limit() {
    let lines = vec!["no braces here", "still nothing"];
    assert_eq!(count_balanced_block_lines(lines.into_iter(), 0), None);
}

#[rstest]
fn balanced_block_unbalanced() {
    let lines = vec!["{", "  open"];
    assert_eq!(count_balanced_block_lines(lines.into_iter(), 10), None);
}

// ---------------------------------------------------------------------------
// is_exempt_symbol
// ---------------------------------------------------------------------------

#[rstest]
#[case("main", true)]
#[case("test_something", true)]
#[case("test_", true)]
#[case("helper", false)]
#[case("main_helper", false)]
#[case("testing", false)]
fn exempt_symbol_check(#[case] name: &str, #[case] expected: bool) {
    assert_eq!(is_exempt_symbol(name), expected, "symbol: {name}");
}

// ---------------------------------------------------------------------------
// count_symbol_occurrences
// ---------------------------------------------------------------------------

#[rstest]
fn symbol_occurrences_basic() {
    let files = vec![
        "fn foo() {} fn bar() { foo() }".to_owned(),
        "fn baz() { foo(); bar() }".to_owned(),
    ];
    let symbols = vec!["foo".to_owned(), "bar".to_owned(), "missing".to_owned()];
    let counts = count_symbol_occurrences(&files, &symbols).expect("should not fail");

    assert_eq!(counts["foo"], 3); // defined + called twice
    assert_eq!(counts["bar"], 2); // defined + called
    assert_eq!(counts["missing"], 0);
}

#[rstest]
fn symbol_occurrences_empty_input() {
    let files: Vec<String> = vec![];
    let symbols = vec!["anything".to_owned()];
    let counts = count_symbol_occurrences(&files, &symbols).expect("should not fail");
    assert_eq!(counts["anything"], 0);
}

// ---------------------------------------------------------------------------
// collect_functions
// ---------------------------------------------------------------------------

#[rstest]
fn collect_functions_finds_pub_async_and_plain() {
    let code = r#"
fn plain() {}
pub fn visible() {}
pub async fn async_visible() {}
async fn async_private() {}
"#;
    let files = vec![(PathBuf::from("test.rs"), code.to_owned())];
    let fns = collect_functions(&files).expect("should not fail");

    let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"plain"));
    assert!(names.contains(&"visible"));
    assert!(names.contains(&"async_visible"));
    assert!(names.contains(&"async_private"));
    assert_eq!(fns.len(), 4);
}

#[rstest]
fn collect_functions_empty_file() {
    let files = vec![(PathBuf::from("empty.rs"), String::new())];
    let fns = collect_functions(&files).expect("should not fail");
    assert!(fns.is_empty());
}

#[rstest]
fn collect_functions_has_correct_line_numbers() {
    let code = "fn first() {}\n\n\nfn second() {}";
    let files = vec![(PathBuf::from("test.rs"), code.to_owned())];
    let fns = collect_functions(&files).expect("should not fail");

    assert_eq!(fns[0].name, "first");
    assert_eq!(fns[0].line, 1);
    assert_eq!(fns[1].name, "second");
    assert_eq!(fns[1].line, 4);
}

// ---------------------------------------------------------------------------
// filter_complex_functions
// ---------------------------------------------------------------------------

#[rstest]
fn filter_complex_below_threshold_excluded() {
    let functions = vec![
        FunctionRecord {
            file: PathBuf::from("a.rs"),
            name: "simple".to_owned(),
            line: 1,
            complexity: 3,
        },
        FunctionRecord {
            file: PathBuf::from("b.rs"),
            name: "complex".to_owned(),
            line: 10,
            complexity: 20,
        },
    ];
    let findings = filter_complex_functions(functions, 5);
    assert_eq!(findings.len(), 1);
    match &findings[0] {
        AnalysisFinding::Complexity {
            function,
            complexity,
            ..
        } => {
            assert_eq!(function, "complex");
            assert_eq!(*complexity, 20);
        }
        other => panic!("expected Complexity, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// detect_dead_functions
// ---------------------------------------------------------------------------

#[rstest]
fn detect_dead_functions_identifies_unused() {
    let functions = vec![
        FunctionRecord {
            file: PathBuf::from("a.rs"),
            name: "used_fn".to_owned(),
            line: 1,
            complexity: 1,
        },
        FunctionRecord {
            file: PathBuf::from("a.rs"),
            name: "unused_fn".to_owned(),
            line: 5,
            complexity: 1,
        },
        FunctionRecord {
            file: PathBuf::from("a.rs"),
            name: "main".to_owned(),
            line: 10,
            complexity: 1,
        },
    ];
    let file_contents = vec![
        "fn used_fn() {} fn main() { used_fn(); }".to_owned(),
        "fn unused_fn() {}".to_owned(),
    ];
    let dead = detect_dead_functions(functions, &file_contents).expect("should not fail");

    let dead_names: Vec<&str> = dead
        .iter()
        .filter_map(|f| match f {
            AnalysisFinding::DeadCode { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert!(dead_names.contains(&"unused_fn"));
    assert!(!dead_names.contains(&"used_fn"));
    assert!(!dead_names.contains(&"main")); // exempt
}

// ---------------------------------------------------------------------------
// compute_tdg_scores
// ---------------------------------------------------------------------------

#[rstest]
fn tdg_scores_above_threshold_returned() {
    let files = vec![(
        PathBuf::from("big.rs"),
        "x\n".repeat(400), // 400 SLOC
    )];
    let functions = vec![FunctionRecord {
        file: PathBuf::from("big.rs"),
        name: "complex".to_owned(),
        line: 1,
        complexity: 20,
    }];
    let dead_code = vec![AnalysisFinding::DeadCode {
        file: PathBuf::from("big.rs"),
        line: 5,
        item_type: "function".to_owned(),
        name: "dead".to_owned(),
    }];

    let scores = compute_tdg_scores(&files, functions, &dead_code, 10);
    assert!(!scores.is_empty(), "should find high TDG file");
    match &scores[0] {
        AnalysisFinding::TechnicalDebt { file, score } => {
            assert_eq!(file, &PathBuf::from("big.rs"));
            assert!(*score > 10);
        }
        other => panic!("expected TechnicalDebt, got {other:?}"),
    }
}

#[rstest]
fn tdg_scores_below_threshold_excluded() {
    let files = vec![(PathBuf::from("small.rs"), "fn tiny() {}".to_owned())];
    let functions = vec![FunctionRecord {
        file: PathBuf::from("small.rs"),
        name: "tiny".to_owned(),
        line: 1,
        complexity: 1,
    }];
    let scores = compute_tdg_scores(&files, functions, &[], 100);
    assert!(scores.is_empty(), "low TDG files should be excluded");
}
