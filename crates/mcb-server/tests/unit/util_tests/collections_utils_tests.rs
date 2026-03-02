use mcb_server::utils::collections::normalize_collection_name;
use rstest::rstest;

#[rstest]
#[test]
fn rejects_too_long_collection_name() {
    let too_long = "a".repeat(256);
    let err = normalize_collection_name(&too_long).expect_err("name must be rejected");
    assert!(err.contains("maximum length"));
}
