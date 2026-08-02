use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
use mcb_domain::utils::tests::utils::{
    create_test_agent_session, create_test_checkpoint, create_test_tool_call,
};
use rstest::{fixture, rstest};

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

#[fixture]
fn tool_call() -> ToolCall {
    create_test_tool_call("tc-001")
}

#[fixture]
fn checkpoint() -> Checkpoint {
    create_test_checkpoint("ckpt-001")
}

#[rstest]
fn test_agent_session_serialization(agent_session: AgentSession) {
    let json = serde_json::to_string(&agent_session).expect("serialize");
    let deserialized: AgentSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "sess-123");
    assert_eq!(deserialized.agent_type, AgentType::Sisyphus);
}

#[rstest]
fn test_delegation_serialization(delegation: Delegation) {
    let json = serde_json::to_string(&delegation).expect("serialize");
    let deserialized: Delegation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, delegation.id);
}

#[rstest]
fn test_tool_call_serialization(tool_call: ToolCall) {
    let json = serde_json::to_string(&tool_call).expect("serialize");
    let deserialized: ToolCall = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, tool_call.id);
}

#[rstest]
fn test_checkpoint_serialization(checkpoint: Checkpoint) {
    let json = serde_json::to_string(&checkpoint).expect("serialize");
    let deserialized: Checkpoint = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, checkpoint.id);
}

#[rstest]
#[case(AgentType::Sisyphus, "sisyphus")]
#[case(AgentType::Oracle, "oracle")]
#[case(AgentType::Explore, "explore")]
fn test_agent_type_as_str(#[case] agent_type: AgentType, #[case] expected: &str) {
    assert_eq!(agent_type.as_str(), expected);
}

#[rstest]
#[case(AgentSessionStatus::Active, "active")]
#[case(AgentSessionStatus::Completed, "completed")]
#[case(AgentSessionStatus::Failed, "failed")]
fn test_agent_session_status_as_str(#[case] status: AgentSessionStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case(CheckpointType::Git, "git")]
#[case(CheckpointType::File, "file")]
#[case(CheckpointType::Config, "config")]
fn test_checkpoint_type_as_str(#[case] checkpoint_type: CheckpointType, #[case] expected: &str) {
    assert_eq!(checkpoint_type.as_str(), expected);
}
