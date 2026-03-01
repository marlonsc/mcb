use mcb_domain::test_utils::TestResult;
use mcb_domain::value_objects::ids::SessionId;
use mcb_server::hooks::{HookProcessor, PostToolUseContext, SessionStartContext};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_hook_processor_creation() -> TestResult {
    let processor = HookProcessor::new(None);
    let err = processor
        .process_post_tool_use(create_test_context()?)
        .await
        .expect_err("hook processor with no memory service should fail");
    assert_eq!(err.to_string(), "Memory service unavailable");
    Ok(())
}

#[rstest]
#[case("test_tool")]
#[case("test")]
#[tokio::test]
async fn test_post_tool_use_hook_graceful_degradation(#[case] tool_name: &str) -> TestResult {
    let processor = HookProcessor::new(None);
    let context = PostToolUseContext::new(tool_name.to_owned(), false)?;

    let result = processor.process_post_tool_use(context).await;
    let err = result.expect_err("post_tool_use with no memory service should fail");
    assert_eq!(err.to_string(), "Memory service unavailable");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_session_start_hook_graceful_degradation() -> TestResult {
    let processor = HookProcessor::new(None);
    let context = SessionStartContext::new(SessionId::from("test_session"))?;

    let result = processor.process_session_start(context).await;
    let err = result.expect_err("session_start with no memory service should fail");
    assert_eq!(err.to_string(), "Memory service unavailable");
    Ok(())
}

#[rstest]
#[case("search", false, false)]
#[case("index", true, false)]
#[case("validate", false, true)]
#[rstest]
#[tokio::test]
async fn test_post_tool_use_context_enrichment(
    #[case] tool_name: &str,
    #[case] with_session_id: bool,
    #[case] with_metadata: bool,
) -> TestResult {
    let mut context = PostToolUseContext::new(tool_name.to_owned(), false)?;

    let session_id_val = SessionId::from("session_123");
    if with_session_id {
        context = context.with_session_id(session_id_val);
    }
    if with_metadata {
        context = context.with_metadata("key".to_owned(), "value".to_owned());
    }

    assert_eq!(context.tool_name, tool_name);
    if with_session_id {
        assert_eq!(
            context
                .session_id
                .as_ref()
                .map(mcb_domain::SessionId::as_str),
            Some(session_id_val.as_str())
        );
    }
    if with_metadata {
        assert_eq!(context.metadata.get("key"), Some(&"value".to_owned()));
    }
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_session_start_context_creation() -> TestResult {
    let sid = SessionId::from("session_456");
    let context = SessionStartContext::new(sid)?;
    assert_eq!(context.session_id.as_str(), sid.as_str());
    assert!(context.timestamp > 0);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_hook_processor_default() -> TestResult {
    let processor = HookProcessor::default();
    let context = PostToolUseContext::new("test".to_owned(), false)?;

    let result = processor.process_post_tool_use(context).await;
    let err = result.expect_err("default hook processor should fail without memory service");
    assert_eq!(err.to_string(), "Memory service unavailable");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_post_tool_use_error_status() -> TestResult {
    let context = PostToolUseContext::new("failing_tool".to_owned(), true)?;

    assert_eq!(context.tool_name, "failing_tool");
    Ok(())
}

fn create_test_context() -> Result<PostToolUseContext, Box<dyn std::error::Error>> {
    Ok(PostToolUseContext::new("test_tool".to_owned(), false)?)
}
