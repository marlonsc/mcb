use mcb_domain::error::Error;
use mcb_server::error_mapping::{to_contextual_tool_error, to_opaque_mcp_error};

#[test]
fn test_to_opaque_mcp_error_not_found() {
    let err = Error::NotFound {
        resource: "test".to_string(),
    };
    let mcp_err = to_opaque_mcp_error(err);
    assert_eq!(mcp_err.message, "Not found: test");
}

#[test]
fn test_to_opaque_mcp_error_internal() {
    let err = Error::Internal {
        message: "secret".to_string(),
    };
    let mcp_err = to_opaque_mcp_error(err);
    assert_eq!(mcp_err.message, "internal server error");
}

#[test]
fn test_to_contextual_tool_error_not_found() {
    let err = Error::NotFound {
        resource: "item".to_string(),
    };
    let result = to_contextual_tool_error(err);
    assert!(result.is_error);
    let content = &result.content[0];
    if let rmcp::model::Content::Text { text } = content {
        assert_eq!(text, "Not found: item");
    } else {
        panic!("Expected text content");
    }
}

#[test]
fn test_to_contextual_tool_error_database() {
    let err = Error::Database {
        message: "db fail".to_string(),
        source: None,
    };
    let result = to_contextual_tool_error(err);
    assert!(result.is_error);
    let content = &result.content[0];
    if let rmcp::model::Content::Text { text } = content {
        assert_eq!(text, "Database error: db fail");
    } else {
        panic!("Expected text content");
    }
}
