#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use mcb_domain::ports::{
        AnalysisFinding, CodeAnalyzer, list_code_analyzers, resolve_code_analyzer,
    };
    use rstest::*;
    use tempfile::TempDir;

    /// Resolve the "native-regex" analyzer from the linkme registry.
    #[fixture]
    fn analyzer() -> Arc<dyn CodeAnalyzer> {
        resolve_code_analyzer("native-regex")
            .expect("native-regex analyzer must be registered in test binary")
    }

    #[test]
    fn native_regex_analyzer_is_registered() {
        let analyzers = list_code_analyzers();
        assert!(
            analyzers.iter().any(|(name, _)| *name == "native-regex"),
            "native-regex not found in registry. Available: {analyzers:?}"
        );
    }

    #[test]
    fn resolve_native_regex_by_name() {
        let result = resolve_code_analyzer("native-regex");
        assert!(result.is_ok(), "Failed to resolve native-regex: {result:?}");
    }

    #[rstest]
    fn detects_high_complexity_functions(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            "
fn simple() { let x = 1; }

fn complex(a: i32) {
    if a > 0 { }
    for _i in 0..10 { }
    while a > 1 { break; }
    match a { 1 => (), _ => () }
}
",
        )?;

        let findings = analyzer.analyze_complexity(temp.path(), 3)?;

        assert_eq!(findings.len(), 1);
        match &findings[0] {
            AnalysisFinding::Complexity { function, .. } => assert_eq!(function, "complex"),
            other => panic!("Expected Complexity finding, got {other:?}"),
        }
        Ok(())
    }

    #[rstest]
    fn detects_dead_code_functions(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            "
fn used() {}
fn dead_fn() {}

fn caller() {
    used();
}
",
        )?;

        let findings = analyzer.detect_dead_code(temp.path())?;

        assert!(
            findings
                .iter()
                .any(|f| matches!(f, AnalysisFinding::DeadCode { name, .. } if name == "dead_fn"))
        );
        Ok(())
    }

    #[rstest]
    fn computes_tdg_score_above_threshold(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            "
fn dead_a() {}
fn dead_b() {}
fn heavy(x: i32) {
    if x > 0 {}
    if x > 1 {}
    if x > 2 {}
    if x > 3 {}
}
",
        )?;

        let findings = analyzer.score_tdg(temp.path(), 15)?;

        assert_eq!(findings.len(), 1);
        match &findings[0] {
            AnalysisFinding::TechnicalDebt { score, .. } => assert!(*score > 15),
            other => panic!("Expected TechnicalDebt finding, got {other:?}"),
        }
        Ok(())
    }

    #[rstest]
    fn empty_workspace_returns_no_findings(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;

        assert!(analyzer.analyze_complexity(temp.path(), 10)?.is_empty());
        assert!(analyzer.detect_dead_code(temp.path())?.is_empty());
        assert!(analyzer.score_tdg(temp.path(), 50)?.is_empty());
        Ok(())
    }
}
