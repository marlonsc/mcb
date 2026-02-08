use mcb_server::constants::*;

#[test]
fn test_jsonrpc_constants_exist() {
    assert_eq!(JSONRPC_METHOD_NOT_FOUND, -32601);
    assert_eq!(JSONRPC_PARSE_ERROR, -32700);
    assert_eq!(JSONRPC_INVALID_REQUEST, -32600);
    assert_eq!(JSONRPC_INVALID_PARAMS, -32602);
    assert_eq!(JSONRPC_INTERNAL_ERROR, -32603);
}

#[test]
fn test_valid_sections_contains_expected() {
    assert!(VALID_SECTIONS.contains(&"server"));
    assert!(VALID_SECTIONS.contains(&"logging"));
}

#[test]
fn test_highlight_names_length() {
    assert_eq!(HIGHLIGHT_NAMES.len(), 13);
}
