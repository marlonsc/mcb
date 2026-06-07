#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use mcb_domain::ports::{AnalysisFinding, CodeAnalyzer};
    use mcb_domain::registry::{list_code_analyzers, resolve_code_analyzer};
    use rstest::{fixture, rstest};
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
        assert!(result.is_ok(), "Failed to resolve native-regex analyzer");
    }

    #[rstest]
    fn detects_high_complexity_functions(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        fs::write(
            temp.path().join("sample.rs"),
            "fn simple() { let x = 1; }\n\
             fn complex(a: i32) {\n\
                if a > 0 { }\n\
                for _i in 0..10 { }\n\
                while a > 1 { break; }\n\
                match a { 1 => (), _ => () }\n\
             }",
        )?;

        let findings = analyzer.analyze_complexity(temp.path(), 3)?;
        assert_eq!(findings.len(), 1);
        assert!(
            matches!(findings[0], AnalysisFinding::Complexity { ref function, .. } if function == "complex")
        );
        Ok(())
    }

    #[rstest]
    fn detects_dead_code_functions(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        fs::write(
            temp.path().join("sample.rs"),
            "fn used() {}\n\
             fn dead_fn() {}\n\
             fn caller() { used(); }",
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
    fn detects_tdg_score_above_threshold(
        analyzer: Arc<dyn CodeAnalyzer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        fs::write(
            temp.path().join("sample.rs"),
            "fn dead_a() {}\n\
             fn dead_b() {}\n\
             fn heavy(x: i32) {\n\
                if x > 0 {}\n\
                if x > 1 {}\n\
                if x > 2 {}\n\
                if x > 3 {}\n\
             }",
        )?;

        let findings = analyzer.score_tdg(temp.path(), 15)?;
        assert_eq!(findings.len(), 1);
        assert!(matches!(findings[0], AnalysisFinding::TechnicalDebt { score, .. } if score > 15));
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
