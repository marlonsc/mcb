use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
use mcb_domain::utils::tests::utils::{
    create_test_agent_session, create_test_checkpoint, create_test_tool_call,
};
use rstest::{fixture, rstest};

#[rstest]
#[case(AgentType::Sisyphus.as_str().to_owned(), "sisyphus")]
#[case(AgentType::Oracle.as_str().to_owned(), "oracle")]
#[case(AgentType::Explore.as_str().to_owned(), "explore")]
#[case(AgentSessionStatus::Active.as_str().to_owned(), "active")]
#[case(AgentSessionStatus::Completed.as_str().to_owned(), "completed")]
#[case(AgentSessionStatus::Failed.as_str().to_owned(), "failed")]
#[case(CheckpointType::Git.as_str().to_owned(), "git")]
#[case(CheckpointType::File.as_str().to_owned(), "file")]
#[case(CheckpointType::Config.as_str().to_owned(), "config")]
fn agent_enums_as_str(#[case] actual: String, #[case] expected: &str) {
    assert_eq!(actual, expected);
}

#[fixture]
fn agent_session() -> AgentSession {
    let mut session = create_test_agent_session("sess-123");
    session.duration_ms = Some(1000000);
    session.status = AgentSessionStatus::Completed;
    session.token_count = Some(1000);
    session.tool_calls_count = Some(5);
    session.delegations_count = Some(2);
    session
}

#[rstest]
fn test_agent_session_serialization(agent_session: AgentSession) {
    let json = serde_json::to_string(&agent_session).expect("serialize");
    let deserialized: AgentSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "sess-123");
    assert_eq!(deserialized.agent_type, AgentType::Sisyphus);
}

#[fixture]
fn delegation() -> Delegation {
    Delegation {
        id: "del-001".to_owned(),
        parent_session_id: "parent-sess".to_owned(),
        child_session_id: "child-sess".to_owned(),
        prompt: "Do something".to_owned(),
        prompt_embedding_id: None,
        result: Some("Done".to_owned()),
        success: true,
        created_at: 1700000000,
        completed_at: Some(1700000500),
        duration_ms: Some(500000),
    }
}

#[rstest]
fn test_delegation_serialization(delegation: Delegation) {
    let json = serde_json::to_string(&delegation).expect("serialize");
    assert!(json.contains("del-001"));
}

#[fixture]
fn tool_call() -> ToolCall {
    create_test_tool_call("tc-001")
}

#[rstest]
fn test_tool_call_serialization(tool_call: ToolCall) {
    let json = serde_json::to_string(&tool_call).expect("serialize");
    assert!(json.contains("tc-001"));
}

#[fixture]
fn checkpoint() -> Checkpoint {
    create_test_checkpoint("ckpt-001")
}

#[rstest]
fn test_checkpoint_serialization(checkpoint: Checkpoint) {
    let json = serde_json::to_string(&checkpoint).expect("serialize");
    assert!(json.contains("ckpt-001"));
}
