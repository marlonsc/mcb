//! Unit tests for Rust rule engine wrapper.

use mcb_validate::engines::ReteEngine;

#[test]
fn test_wrapper_creation() {
    let _wrapper = ReteEngine::new();
}

#[test]
fn test_wrapper_clone() {
    let wrapper = ReteEngine::new();
    let _cloned = wrapper.clone();
}
