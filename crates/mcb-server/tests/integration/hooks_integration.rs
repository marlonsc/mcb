use mcb_domain::value_objects::ids::SessionId;
use mcb_server::hooks::{HookProcessor, PostToolUseContext, SessionStartContext};
use rmcp::model::{CallToolResult, Content};
use rstest::rstest;

#[tokio::test]
async fn test_hook_processor_creation() {
    let processor = HookProcessor::new(None);
    assert!(
        processor
            .process_post_tool_use(create_test_context())
            .await
            .is_err()
    );
}

#[rstest]
#[case("test_tool", "Test output")]
#[case("test", "")]
#[tokio::test]
async fn test_post_tool_use_hook_graceful_degradation(
    #[case] tool_name: &str,
    #[case] output: &str,
) {
    let processor = HookProcessor::new(None);
    let context = PostToolUseContext::new(
        tool_name.to_string(),
        if output.is_empty() {
            CallToolResult::success(vec![])
        } else {
            CallToolResult::success(vec![Content::text(output)])
        },
    );

    let result = processor.process_post_tool_use(context).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Memory service unavailable"
    );
}

#[tokio::test]
async fn test_session_start_hook_graceful_degradation() {
    let processor = HookProcessor::new(None);
    let context = SessionStartContext::new(SessionId::new("test_session"));

    let result = processor.process_session_start(context).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Memory service unavailable"
    );
}

#[rstest]
#[case("search", false, false)]
#[case("index", true, false)]
#[case("validate", false, true)]
#[tokio::test]
async fn test_post_tool_use_context_enrichment(
    #[case] tool_name: &str,
    #[case] with_session_id: bool,
    #[case] with_metadata: bool,
) {
    let tool_output = CallToolResult::success(vec![Content::text("Output")]);
    let mut context = PostToolUseContext::new(tool_name.to_string(), tool_output);

    if with_session_id {
        context = context.with_session_id(SessionId::new("session_123"));
    }
    if with_metadata {
        context = context.with_metadata("key".to_string(), "value".to_string());
    }

    assert_eq!(context.tool_name, tool_name);
    if with_session_id {
        assert_eq!(
            context.session_id.as_ref().map(|id| id.as_str()),
            Some("session_123")
        );
    }
    if with_metadata {
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }
}

#[tokio::test]
async fn test_session_start_context_creation() {
    let context = SessionStartContext::new(SessionId::new("session_456"));
    assert_eq!(context.session_id.as_str(), "session_456");
    assert!(context.timestamp > 0);
}

#[tokio::test]
async fn test_hook_processor_default() {
    let processor = HookProcessor::default();
    let context = PostToolUseContext::new("test".to_string(), CallToolResult::success(vec![]));

    let result = processor.process_post_tool_use(context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_post_tool_use_error_status() {
    let tool_output = CallToolResult::error(vec![Content::text("Error occurred")]);
    let context = PostToolUseContext::new("failing_tool".to_string(), tool_output);

    assert_eq!(context.tool_name, "failing_tool");
}

fn create_test_context() -> PostToolUseContext {
    PostToolUseContext::new(
        "test_tool".to_string(),
        CallToolResult::success(vec![Content::text("Test")]),
    )
}
