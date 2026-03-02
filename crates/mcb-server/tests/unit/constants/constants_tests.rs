use mcb_utils::constants::protocol::{JSONRPC_INTERNAL_ERROR, JSONRPC_PARSE_ERROR};
use rstest::rstest;

#[rstest]
#[case(JSONRPC_PARSE_ERROR, -32700)]
#[case(JSONRPC_INTERNAL_ERROR, -32603)]
fn test_jsonrpc_constants_exist(#[case] actual: i32, #[case] expected: i32) {
    assert_eq!(actual, expected);
}

#[rstest]
#[test]
fn test_highlight_names_length() {
    assert_eq!(mcb_domain::value_objects::browse::HIGHLIGHT_NAMES.len(), 13);
}
