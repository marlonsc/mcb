//! Unit tests for Rust rule engine wrapper.
//!
//! Moved from inline tests in src/engines/rust_rule_engine.rs.

use mcb_validate::engines::RustRuleEngineWrapper;

#[test]
fn test_wrapper_creation() {
    let _wrapper = RustRuleEngineWrapper::new();
}

#[test]
fn test_wrapper_clone() {
    let wrapper = RustRuleEngineWrapper::new();
    let _cloned = wrapper.clone();
}
