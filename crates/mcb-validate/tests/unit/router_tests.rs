//! Unit tests for `RuleEngineRouter`.
//!
//! Uses `ENGINE_NAME_*` constants instead of hardcoded engine strings,
//! and `DOMAIN_CRATE` / `FORBIDDEN_PREFIX_PATTERN` from shared constants.

use mcb_validate::engines::{RoutedEngine, RuleEngineRouter};
use serde_json::json;

use crate::test_constants::*;

#[test]
fn test_detect_rete_engine_explicit() {
    let router = RuleEngineRouter::new();

    let rule = json!({
        "engine": ENGINE_NAME_RUST_RULE,
        "rule": "rule Test { when true then Action(); }"
    });

    assert_eq!(router.detect_engine(&rule), RoutedEngine::Rete);
}

#[test]
fn test_detect_rete_engine_by_content() {
    let router = RuleEngineRouter::new();

    let rule = json!({
        "rule": format!(r#"
            rule DomainCheck "Check domain" {{
                when
                    Crate(name == "{DOMAIN_CRATE}")
                then
                    Violation("Error");
            }}
        "#)
    });

    assert_eq!(router.detect_engine(&rule), RoutedEngine::Rete);
}

#[test]
fn test_detect_expression_engine() {
    let router = RuleEngineRouter::new();

    let rule = json!({
        "expression": "file_count > 100",
        "message": "Too many files"
    });

    assert_eq!(router.detect_engine(&rule), RoutedEngine::Expression);
}

#[test]
fn test_detect_rusty_rules_engine() {
    let router = RuleEngineRouter::new();

    let rule = json!({
        "condition": {
            "all": [
                { "fact_type": "file", "field": "path", "operator": "matches", "value": "*.rs" }
            ]
        },
        "action": {
            "violation": { "message": "Rule triggered" }
        }
    });

    assert_eq!(router.detect_engine(&rule), RoutedEngine::RustyRules);
}

#[test]
fn test_detect_default_engine() {
    let router = RuleEngineRouter::new();

    let rule = json!({
        "type": "cargo_dependencies",
        "pattern": FORBIDDEN_PREFIX_PATTERN
    });

    assert_eq!(router.detect_engine(&rule), RoutedEngine::RustyRules);
}

#[test]
fn test_validate_rete_rule() {
    let router = RuleEngineRouter::new();

    let valid_rule = json!({
        "engine": ENGINE_NAME_RETE,
        "rule": "rule Test { when true then Action(); }"
    });
    assert!(router.validate_rule(&valid_rule).is_ok());

    let invalid_rule = json!({
        "engine": ENGINE_NAME_RETE,
        "message": "Something"
    });
    assert!(router.validate_rule(&invalid_rule).is_err());
}

#[test]
fn test_validate_expression_rule() {
    let router = RuleEngineRouter::new();

    let valid_rule = json!({
        "engine": ENGINE_NAME_EXPRESSION,
        "expression": "x > 5"
    });
    assert!(router.validate_rule(&valid_rule).is_ok());

    let invalid_rule = json!({
        "engine": ENGINE_NAME_EXPRESSION,
        "message": "Something"
    });
    assert!(router.validate_rule(&invalid_rule).is_err());
}
