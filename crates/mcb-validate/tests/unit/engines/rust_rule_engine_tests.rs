//! Unit tests for Rust rule engine wrapper.

use mcb_validate::engines::ReteEngine;
use rstest::rstest;

#[rstest]
#[test]
fn test_wrapper_creation() {
    let _wrapper = ReteEngine::new();
}

#[rstest]
#[test]
fn test_wrapper_clone() {
    let wrapper = ReteEngine::new();
    let _cloned = wrapper.clone();
}
