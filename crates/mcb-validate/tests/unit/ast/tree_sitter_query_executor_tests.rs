use mcb_validate::ValidationError;
use mcb_validate::ast::TreeSitterQueryExecutor;
use mcb_validate::rules::yaml_loader::ValidatedRule;
use rstest::rstest;
use serde_json::json;
use tempfile::TempDir;

fn build_rule(query: Option<&str>) -> ValidatedRule {
    ValidatedRule {
        id: "AST_EXEC_001".to_owned(),
        name: "AST Exec".to_owned(),
        category: "quality".to_owned(),
        severity: "warning".to_owned(),
        enabled: true,
        description: "AST test rule".to_owned(),
        rationale: "test".to_owned(),
        engine: "regex".to_owned(),
        config: json!({"patterns": {"placeholder": "fn\\s+"}}),
        rule_definition: json!({}),
        fixes: Vec::new(),
        lint_select: Vec::new(),
        message: None,
        selectors: Vec::new(),
        ast_query: query.map(str::to_owned),
        metrics: None,
        filters: None,
    }
}

#[rstest]
#[test]
fn tree_sitter_query_executor_matches_function_names() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("sample.rs");
    std::fs::write(&file, "fn alpha() {}\nfn beta() {}\nfn gamma() {}\n").unwrap();

    let rule = build_rule(Some("(function_item name: (identifier) @name)"));
    let matches = TreeSitterQueryExecutor::execute(&rule, &file).unwrap();

    assert_eq!(matches.len(), 3);
    assert!(matches.iter().all(|m| m.capture_name == "name"));
    assert!(matches.iter().all(|m| m.node_kind == "identifier"));
    assert_eq!(matches[0].line, 1);
}

#[rstest]
#[test]
fn tree_sitter_query_executor_returns_config_error_for_invalid_query() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("sample.rs");
    std::fs::write(&file, "fn alpha() {}\n").unwrap();

    let rule = build_rule(Some("(function_item name: (identifier) @name"));
    let result = TreeSitterQueryExecutor::execute(&rule, &file);

    match result {
        Err(ValidationError::Config(message)) => {
            assert!(message.contains("Invalid tree-sitter query"));
        }
        Err(other) => panic!("expected ValidationError::Config, got {other:?}"),
        Ok(_) => panic!("expected query compilation to fail"),
    }
}
