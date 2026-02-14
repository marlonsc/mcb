use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
use rstest::rstest;

#[rstest]
#[case(AgentType::Sisyphus.as_str().to_string(), "sisyphus")]
#[case(AgentType::Oracle.as_str().to_string(), "oracle")]
#[case(AgentType::Explore.as_str().to_string(), "explore")]
#[case(AgentSessionStatus::Active.as_str().to_string(), "active")]
#[case(AgentSessionStatus::Completed.as_str().to_string(), "completed")]
#[case(AgentSessionStatus::Failed.as_str().to_string(), "failed")]
#[case(CheckpointType::Git.as_str().to_string(), "git")]
#[case(CheckpointType::File.as_str().to_string(), "file")]
#[case(CheckpointType::Config.as_str().to_string(), "config")]
#[test]
fn test_agent_enums_as_str(#[case] actual: String, #[case] expected: &str) {
    assert_eq!(actual, expected);
}

#[test]
fn test_agent_session_serialization() {
    let session = AgentSession {
        id: "sess-123".to_string(),
        session_summary_id: "sum-456".to_string(),
        agent_type: AgentType::Sisyphus,
        model: "claude-sonnet".to_string(),
        parent_session_id: None,
        started_at: 1700000000,
        ended_at: Some(1700001000),
        duration_ms: Some(1000000),
        status: AgentSessionStatus::Completed,
        prompt_summary: Some("Test prompt".to_string()),
        result_summary: Some("Success".to_string()),
        token_count: Some(1000),
        tool_calls_count: Some(5),
        delegations_count: Some(2),
        project_id: None,
        worktree_id: None,
    };

    let json = serde_json::to_string(&session).expect("serialize");
    let deserialized: AgentSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "sess-123");
    assert_eq!(deserialized.agent_type, AgentType::Sisyphus);
}

#[test]
fn test_delegation_serialization() {
    let delegation = Delegation {
        id: "del-001".to_string(),
        parent_session_id: "parent-sess".to_string(),
        child_session_id: "child-sess".to_string(),
        prompt: "Do something".to_string(),
        prompt_embedding_id: None,
        result: Some("Done".to_string()),
        success: true,
        created_at: 1700000000,
        completed_at: Some(1700000500),
        duration_ms: Some(500000),
    };

    let json = serde_json::to_string(&delegation).expect("serialize");
    assert!(json.contains("del-001"));
}

#[test]
fn test_tool_call_serialization() {
    let tool_call = ToolCall {
        id: "tc-001".to_string(),
        session_id: "sess-123".to_string(),
        tool_name: "read_file".to_string(),
        params_summary: Some("path=/foo".to_string()),
        success: true,
        error_message: None,
        duration_ms: Some(100),
        created_at: 1700000000,
    };

    let json = serde_json::to_string(&tool_call).expect("serialize");
    assert!(json.contains("read_file"));
}

#[test]
fn test_checkpoint_serialization() {
    let checkpoint = Checkpoint {
        id: "ckpt-001".to_string(),
        session_id: "sess-123".to_string(),
        checkpoint_type: CheckpointType::Git,
        description: "Before risky operation".to_string(),
        snapshot_data: serde_json::json!({"files": ["a.rs", "b.rs"]}),
        created_at: 1700000000,
        restored_at: None,
        expired: false,
    };

    let json = serde_json::to_string(&checkpoint).expect("serialize");
    assert!(json.contains("ckpt-001"));
}
