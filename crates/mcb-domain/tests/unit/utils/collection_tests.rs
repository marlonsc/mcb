//! Unit tests for test collection name helpers.

use mcb_domain::utils::tests::collection::unique_collection;

#[test]
fn unique_collection_format_has_prefix() {
    let name = unique_collection("mypfx");
    assert!(name.starts_with("test_mypfx_"), "unexpected format: {name}");
}

#[test]
fn unique_collection_names_differ_between_calls() {
    let a = unique_collection("col");
    let b = unique_collection("col");
    assert_ne!(a, b, "successive calls must return distinct names");
}
