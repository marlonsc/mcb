//! Tests for RETE engine.
//!
//! Uses shared helpers:
//! - `build_grl_rule` — eliminates inline GRL templates
//! - `build_kb_from_grl` — eliminates repeated KB+parser boilerplate
//! - `create_facts` — eliminates repeated Facts setup
//! - `FACT_*` constants — eliminates hardcoded fact key strings

use mcb_validate::engines::rete_engine::ReteEngine;
use rust_rule_engine::{RustRuleEngine, Value as RreValue};

use crate::utils::test_constants::{
    DOMAIN_CRATE, FACT_CRATE_NAME, FACT_HAS_INTERNAL_DEPS, FACT_RESULT_VALUE,
    FACT_VIOLATION_MESSAGE, FACT_VIOLATION_RULE_NAME, FACT_VIOLATION_TRIGGERED, FORBIDDEN_PREFIX,
    RULE_CA001,
};
use crate::utils::*;
use rstest::rstest;

#[rstest]
#[test]
fn test_rete_engine_creation() {
    let _engine = ReteEngine::new();
}

/// Verifies that GRL parsing works with our syntax.
#[rstest]
#[tokio::test]
async fn test_grl_parsing_with_assertion() {
    let mut engine = ReteEngine::new();

    let grl = build_grl_rule(
        "TestRule",
        &format!("{FACT_HAS_INTERNAL_DEPS} == true"),
        &format!("{FACT_VIOLATION_TRIGGERED} = true"),
    );

    let result = engine.load_grl(&grl);
    assert!(
        result.is_ok(),
        "GRL parsing FAILED: {:?}. This means rust-rule-engine doesn't accept our syntax!",
        result.err()
    );
}

/// Verifies that rules fire and modify facts.
#[rstest]
#[tokio::test]
async fn test_rule_execution_modifies_facts() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = ReteEngine::new();

    let grl = build_grl_rule(
        "TestRule",
        &format!("{FACT_HAS_INTERNAL_DEPS} == true"),
        &format!("{FACT_RESULT_VALUE} = true"),
    );

    engine.load_grl(&grl)?;

    let kb = build_kb_from_grl("test", &grl)?;
    let facts = create_facts(&[(FACT_HAS_INTERNAL_DEPS, RreValue::Boolean(true))]);

    let mut rre_engine = RustRuleEngine::new(kb);
    let result = rre_engine.execute(&facts)?;
    assert!(
        result.rules_fired > 0,
        "No rules fired! Expected at least 1 rule to fire. Got: rules_fired={}, rules_evaluated={}",
        result.rules_fired,
        result.rules_evaluated
    );

    assert_eq!(
        facts.get(FACT_RESULT_VALUE),
        Some(RreValue::Boolean(true)),
        "Rule did NOT modify the fact! {FACT_RESULT_VALUE} should be Boolean(true)"
    );
    Ok(())
}

/// End-to-end test for CA001 Domain Independence rule:
/// YAML rule → GRL parsing → execution → violation detection.
#[rstest]
#[tokio::test]
async fn test_ca001_detects_violation_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    let grl = build_grl_rule(
        "DomainIndependence",
        &format!("{FACT_HAS_INTERNAL_DEPS} == true && {FACT_VIOLATION_TRIGGERED} == false"),
        &format!(
            "{FACT_VIOLATION_TRIGGERED} = true;\n        \
             {FACT_VIOLATION_MESSAGE} = \"Domain layer cannot depend on internal {FORBIDDEN_PREFIX}* crates\";\n        \
             {FACT_VIOLATION_RULE_NAME} = \"{RULE_CA001}\""
        ),
    );

    let kb = build_kb_from_grl("test", &grl)?;

    // Case 1: VIOLATION — has_internal_dependencies=true
    {
        let facts = create_facts(&[
            (FACT_CRATE_NAME, RreValue::String(DOMAIN_CRATE.to_owned())),
            (FACT_HAS_INTERNAL_DEPS, RreValue::Boolean(true)),
            (FACT_VIOLATION_TRIGGERED, RreValue::Boolean(false)),
        ]);

        let mut rre_engine = RustRuleEngine::new(kb.clone());
        let result = rre_engine.execute(&facts)?;

        assert_eq!(
            result.rules_fired, 1,
            "CA001 should fire when has_internal_dependencies=true! rules_fired={}",
            result.rules_fired
        );

        assert_eq!(
            facts.get(FACT_VIOLATION_TRIGGERED),
            Some(RreValue::Boolean(true)),
            "CA001 did not trigger violation"
        );
    }

    // Case 2: NO violation — has_internal_dependencies=false
    {
        let facts = create_facts(&[
            (FACT_CRATE_NAME, RreValue::String(DOMAIN_CRATE.to_owned())),
            (FACT_HAS_INTERNAL_DEPS, RreValue::Boolean(false)),
            (FACT_VIOLATION_TRIGGERED, RreValue::Boolean(false)),
        ]);

        let mut rre_engine = RustRuleEngine::new(kb);
        let result = rre_engine.execute(&facts)?;

        assert_eq!(
            result.rules_fired, 0,
            "CA001 should NOT fire when has_internal_dependencies=false! rules_fired={}",
            result.rules_fired
        );

        assert_eq!(
            facts.get(FACT_VIOLATION_TRIGGERED),
            Some(RreValue::Boolean(false)),
            "CA001 incorrectly triggered violation"
        );
    }
    Ok(())
}
