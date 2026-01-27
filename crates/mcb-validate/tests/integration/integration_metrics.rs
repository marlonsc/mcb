//! Integration tests for Phase 4: Complexity Metrics
//!
//! Tests the full flow: source file → `RcaAnalyzer` → violations
//! Using rust-code-analysis (RCA) directly - NO wrappers.

#[cfg(test)]
mod integration_metrics_tests {
    use mcb_validate::metrics::{MetricThresholds, MetricType, RcaAnalyzer};
    use mcb_validate::violation_trait::Severity;
    use rust_code_analysis::LANG;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test analyzing a simple function that should pass all thresholds
    #[test]
    fn test_simple_function_passes() {
        let analyzer = RcaAnalyzer::new();
        let path = Path::new("simple.rs");

        let content = br"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

        let functions = analyzer.analyze_code(content, &LANG::Rust, path).unwrap();

        // Simple function should have low complexity
        for func in &functions {
            assert!(
                func.metrics.cognitive <= 5.0,
                "Simple function should have low cognitive complexity"
            );
        }
    }

    /// Test detecting high cognitive complexity
    #[test]
    fn test_detects_high_cognitive_complexity() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            5,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                for i in 0..x {
                    if i % 2 == 0 {
                        return i;
                    }
                }
            }
        }
    }
    x
}
";

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("complex.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        assert!(
            !violations.is_empty(),
            "Should detect cognitive complexity violation"
        );

        let complexity_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::CognitiveComplexity)
            .collect();

        assert!(!complexity_violations.is_empty());
        assert_eq!(complexity_violations[0].item_name, "complex");
    }

    /// Test detecting high cyclomatic complexity
    #[test]
    fn test_detects_high_cyclomatic_complexity() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CyclomaticComplexity,
            3,
            Severity::Error,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        match x {
            1 => 1,
            2 => 2,
            3 => 3,
            _ => x
        }
    } else {
        0
    }
}
";

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("branchy.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        let cyclomatic_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::CyclomaticComplexity)
            .collect();

        assert!(
            !cyclomatic_violations.is_empty(),
            "Should detect cyclomatic complexity violation"
        );
        assert_eq!(cyclomatic_violations[0].severity, Severity::Error);
    }

    /// Test detecting long functions
    #[test]
    fn test_detects_long_function() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::FunctionLength,
            5,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br"
fn long_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
}
";

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("long.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        let length_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::FunctionLength)
            .collect();

        assert!(
            !length_violations.is_empty(),
            "Should detect function length violation"
        );
    }

    /// Test analyzing impl block methods
    #[test]
    fn test_analyzes_impl_methods() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br"
struct Calculator;

impl Calculator {
    fn simple(&self) -> i32 {
        1 + 1
    }

    fn complex(&self, x: i32) -> i32 {
        if x > 0 {
            if x > 10 {
                if x > 100 {
                    return x * 2;
                }
            }
        }
        x
    }
}
";

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("impl.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        // Should detect complex but not simple
        let names: Vec<_> = violations.iter().map(|v| &v.item_name).collect();
        assert!(names.contains(&&"complex".to_string()));
        assert!(!names.contains(&&"simple".to_string()));
    }

    /// Test analyzing a real file
    #[test]
    fn test_analyze_real_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        std::fs::write(
            &file_path,
            r"
fn test_function(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => 1,
        _ => x * 2,
    }
}
",
        )
        .unwrap();

        let analyzer = RcaAnalyzer::new();
        let result = analyzer.analyze_file(&file_path);

        // Should complete without error
        assert!(result.is_ok(), "analyze_file should succeed");
    }

    /// Test multiple files analysis
    #[test]
    fn test_analyze_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");

        std::fs::write(&file1, "fn simple() { let x = 1; }").unwrap();
        std::fs::write(
            &file2,
            r#"
fn complex(x: i32) {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                println!("nested");
            }
        }
    }
}
"#,
        )
        .unwrap();

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        // Analyze each file
        let violations1 = analyzer.find_violations(&file1).unwrap();
        let violations2 = analyzer.find_violations(&file2).unwrap();

        // Should find complexity in file2 but not file1
        assert!(violations1.is_empty(), "file1 should have no violations");
        assert!(!violations2.is_empty(), "file2 should have violations");
    }

    /// Test thresholds from YAML config
    #[test]
    fn test_thresholds_from_yaml_config() {
        let yaml = serde_json::json!({
            "cognitive_complexity": {
                "max": 10,
                "severity": "error"
            },
            "function_length": {
                "max": 30,
                "severity": "warning"
            },
            "nesting_depth": {
                "max": 3
            }
        });

        let thresholds = MetricThresholds::from_yaml(&yaml);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 10);
        assert_eq!(cc.severity, Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 30);
        assert_eq!(fl.severity, Severity::Warning);

        let nd = thresholds.get(MetricType::NestingDepth).unwrap();
        assert_eq!(nd.max_value, 3);
        assert_eq!(nd.severity, Severity::Warning); // Default
    }

    /// Test violation message format
    #[test]
    fn test_violation_message_format() {
        // Use threshold of 0 so any if/for/while triggers a violation
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            0,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br#"
fn with_if(x: i32) {
    if x > 0 {
        println!("positive");
    }
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("msg.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        assert!(
            !violations.is_empty(),
            "Should have violations with threshold=0"
        );
        let v = &violations[0];

        // Check violation fields
        assert_eq!(v.item_name, "with_if");
        assert_eq!(v.metric_type, MetricType::CognitiveComplexity);
        assert!(v.actual_value >= 1); // Should be at least 1 (the if statement)
        assert_eq!(v.threshold, 0);
    }

    /// Test suggestion text via Violation trait
    #[test]
    fn test_suggestion_text() {
        use mcb_validate::violation_trait::Violation;

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CyclomaticComplexity,
            1,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = br#"
fn nested(x: i32) {
    if x > 0 {
        if x > 10 {
            println!("nested");
        }
    }
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("suggestion.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        assert!(!violations.is_empty());
        let v = &violations[0];

        // Check suggestion is provided via Violation trait
        let suggestion = v.suggestion();
        assert!(suggestion.is_some());
        let suggestion_text = suggestion.unwrap();
        assert!(!suggestion_text.is_empty());
    }

    /// Test multi-language support
    #[test]
    fn test_python_analysis() {
        let analyzer = RcaAnalyzer::new();
        let path = Path::new("test.py");

        let content = br"
def complex_function(x):
    if x > 0:
        if x > 10:
            if x > 100:
                return x * 2
    return x
";

        let functions = analyzer.analyze_code(content, &LANG::Python, path).unwrap();

        assert!(!functions.is_empty(), "Should find Python functions");
    }

    /// Test JavaScript analysis
    #[test]
    fn test_javascript_analysis() {
        let analyzer = RcaAnalyzer::new();
        let path = Path::new("test.js");

        let content = br"
function complexFunction(x) {
    if (x > 0) {
        if (x > 10) {
            if (x > 100) {
                return x * 2;
            }
        }
    }
    return x;
}
";

        let functions = analyzer.analyze_code(content, &LANG::Mozjs, path).unwrap();

        assert!(!functions.is_empty(), "Should find JavaScript functions");
    }
}
