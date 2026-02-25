use mcb_server::constants::*;
use rstest::rstest;

#[rstest]
#[case(JSONRPC_METHOD_NOT_FOUND, -32601)]
#[case(JSONRPC_PARSE_ERROR, -32700)]
#[case(JSONRPC_INVALID_REQUEST, -32600)]
#[case(JSONRPC_INVALID_PARAMS, -32602)]
#[case(JSONRPC_INTERNAL_ERROR, -32603)]
fn test_jsonrpc_constants_exist(#[case] actual: i32, #[case] expected: i32) {
    assert_eq!(actual, expected);
}

#[test]
fn test_highlight_names_length() {
    assert_eq!(
        mcb_infrastructure::constants::highlight::HIGHLIGHT_NAMES.len(),
        13
    );
}
