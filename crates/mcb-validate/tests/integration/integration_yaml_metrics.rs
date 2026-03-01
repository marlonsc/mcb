//! Integration tests for YAML Metrics Rules (Phase 4)
//!
//! Tests the full pipeline: YAML rule → `MetricsConfig` → `RcaAnalyzer` → violations
//! Using rust-code-analysis (RCA) directly - NO wrappers.

#[cfg(test)]
mod yaml_metrics_tests {
    use mcb_validate::Severity;
    use mcb_validate::metrics::{MetricThresholds, MetricType, RcaAnalyzer};
    use mcb_validate::rules::yaml_loader::{MetricThresholdConfig, MetricsConfig, YamlRuleLoader};
    use rstest::rstest;
    use tempfile::TempDir;

    /// Test that `MetricsConfig` can be converted to `MetricThresholds`
    #[rstest]
    #[test]
    fn test_metrics_config_to_thresholds() {
        let config = MetricsConfig {
            cognitive_complexity: Some(MetricThresholdConfig {
                max: 10,
                severity: Some("error".to_owned()),
                languages: Some(vec!["rust".to_owned()]),
            }),
            cyclomatic_complexity: None,
            function_length: Some(MetricThresholdConfig {
                max: 30,
                severity: Some("warning".to_owned()),
                languages: None,
            }),
            nesting_depth: Some(MetricThresholdConfig {
                max: 3,
                severity: None, // Defaults to warning
                languages: None,
            }),
        };

        let thresholds = MetricThresholds::from_metrics_config(&config);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 10);
        assert_eq!(cc.severity, Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 30);
        assert_eq!(fl.severity, Severity::Warning);

        let nd = thresholds.get(MetricType::NestingDepth).unwrap();
        assert_eq!(nd.max_value, 3);
        assert_eq!(nd.severity, Severity::Warning);
    }

    /// Test analyzing code with thresholds from `MetricsConfig`
    #[rstest]
    #[test]
    fn test_analyze_with_metrics_config() {
        let config = MetricsConfig {
            cognitive_complexity: Some(MetricThresholdConfig {
                max: 3,
                severity: Some("error".to_owned()),
                languages: None,
            }),
            cyclomatic_complexity: None,
            function_length: None,
            nesting_depth: None,
        };

        let thresholds = MetricThresholds::from_metrics_config(&config);
        let analyzer = RcaAnalyzer::with_thresholds(thresholds);

        let content = b"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
        }
    }
    x
}
";

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, content).unwrap();

        let violations = analyzer.find_violations(&file_path).unwrap();

        assert!(!violations.is_empty(), "Should detect complexity violation");
        let v = &violations[0];
        assert_eq!(v.item_name, "complex");
        assert_eq!(v.metric_type, MetricType::CognitiveComplexity);
        assert_eq!(v.severity, Severity::Error);
    }

    /// Test loading YAML rules with and without metrics section
    #[rstest]
    #[case(
        "METRIC001.yml",
        r#"
schema: "rule/v3"
id: "METRIC001"
name: "Cognitive Complexity Limit"
category: "metrics"
severity: "warning"
enabled: true
description: "Enforces a maximum cognitive complexity limit for functions"
rationale: "High cognitive complexity makes code harder to understand"

metrics:
  cognitive_complexity:
    max: 5
    severity: warning
    languages: ["rust"]
  nesting_depth:
    max: 3
    severity: error
"#,
        "METRIC001",
        "metrics",
        true
    )]
    #[case(
        "QUAL001.yml",
        r#"
schema: "rule/v2"
id: "QUAL001"
name: "Test Rule"
category: "quality"
severity: "warning"
enabled: true
description: "A test rule without metrics"
rationale: "For testing"

lint_select: ["F401"]
"#,
        "QUAL001",
        "quality",
        false
    )]
    #[rstest]
    #[tokio::test]
    async fn load_yaml_rule_cases(
        #[case] file_name: &str,
        #[case] rule_yaml: &str,
        #[case] expected_id: &str,
        #[case] expected_category: &str,
        #[case] expect_metrics: bool,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        std::fs::write(rules_dir.join(file_name), rule_yaml).unwrap();

        let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
        let rules = loader.load_all_rules().await.unwrap();

        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.id, expected_id);
        assert_eq!(rule.category, expected_category);

        assert_eq!(rule.metrics.is_some(), expect_metrics);
        if expect_metrics {
            let metrics = rule.metrics.as_ref().unwrap();
            assert!(metrics.cognitive_complexity.is_some());
            let cc = metrics.cognitive_complexity.as_ref().unwrap();
            assert_eq!(cc.max, 5);

            assert!(metrics.nesting_depth.is_some());
            let nd = metrics.nesting_depth.as_ref().unwrap();
            assert_eq!(nd.max, 3);
        }
    }

    /// Test full pipeline: YAML rule → `MetricThresholds` → `RcaAnalyzer` → violations
    #[rstest]
    #[tokio::test]
    async fn test_full_yaml_metrics_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        // Create a metrics rule YAML
        let rule_yaml = r#"
schema: "rule/v3"
id: "METRIC001"
name: "Cognitive Complexity Limit"
category: "metrics"
severity: "warning"
enabled: true
description: "Enforces complexity limits"
rationale: "Keep code simple"

metrics:
  cognitive_complexity:
    max: 2
    severity: error
"#;

        std::fs::write(rules_dir.join("METRIC001.yml"), rule_yaml).unwrap();

        // Load the rule
        let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
        let rules = loader.load_all_rules().await.unwrap();
        let rule = &rules[0];

        // Convert to thresholds
        let metrics_config = rule.metrics.as_ref().unwrap();
        let thresholds = MetricThresholds::from_metrics_config(metrics_config);

        // Analyze code
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

        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, content).unwrap();

        let violations = analyzer.find_violations(&test_file).unwrap();

        // Should detect cognitive complexity > 2
        assert!(!violations.is_empty(), "Should detect violation");
        let v = &violations[0];
        assert!(v.actual_value > 2, "Actual value should exceed threshold");
    }
}
