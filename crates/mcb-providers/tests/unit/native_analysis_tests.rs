use std::fs;

use mcb_domain::ports::providers::{ComplexityAnalyzer, DeadCodeDetector, TdgScorer};
use mcb_providers::analysis::NativePmatAnalyzer;
use rstest::*;
use tempfile::TempDir;

#[fixture]
fn analyzer() -> NativePmatAnalyzer {
    NativePmatAnalyzer
}

#[rstest]
fn detects_high_complexity_functions(analyzer: NativePmatAnalyzer) {
    let temp = TempDir::new().expect("create tempdir");
    let file = temp.path().join("sample.rs");
    fs::write(
        &file,
        r#"
fn simple() { let x = 1; }

fn complex(a: i32) {
    if a > 0 { }
    for _i in 0..10 { }
    while a > 1 { break; }
    match a { 1 => (), _ => () }
}
"#,
    )
    .expect("write sample");

    let findings = analyzer
        .analyze_complexity(temp.path(), 3)
        .expect("analyze complexity");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].function, "complex");
}

#[rstest]
fn detects_dead_code_functions(analyzer: NativePmatAnalyzer) {
    let temp = TempDir::new().expect("create tempdir");
    let file = temp.path().join("sample.rs");
    fs::write(
        &file,
        r#"
fn used() {}
fn dead_fn() {}

fn caller() {
    used();
}
"#,
    )
    .expect("write sample");

    let findings = analyzer
        .detect_dead_code(temp.path())
        .expect("detect dead code");

    assert!(findings.iter().any(|f| f.name == "dead_fn"));
}

#[rstest]
fn computes_tdg_score_above_threshold(analyzer: NativePmatAnalyzer) {
    let temp = TempDir::new().expect("create tempdir");
    let file = temp.path().join("sample.rs");
    fs::write(
        &file,
        r#"
fn dead_a() {}
fn dead_b() {}
fn heavy(x: i32) {
    if x > 0 {}
    if x > 1 {}
    if x > 2 {}
    if x > 3 {}
}
"#,
    )
    .expect("write sample");

    let findings = analyzer.score_tdg(temp.path(), 15).expect("score tdg");

    assert_eq!(findings.len(), 1);
    assert!(findings[0].score > 15);
}
