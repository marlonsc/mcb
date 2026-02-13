use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};

#[test]
// TODO(TEST003): Bad test function name 'worktree_status_as_str'. Use 'test_worktree_status_as_str'.
fn worktree_status_as_str() {
    assert_eq!(WorktreeStatus::Active.as_str(), "active");
    assert_eq!(WorktreeStatus::InUse.as_str(), "in_use");
    assert_eq!(WorktreeStatus::Pruned.as_str(), "pruned");
}

#[test]
// TODO(TEST003): Bad test function name 'worktree_status_from_str'. Use 'test_worktree_status_from_str'.
fn worktree_status_from_str() {
    assert_eq!(
        "active".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::Active)
    );
    assert_eq!(
        "in_use".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::InUse)
    );
    assert_eq!(
        "pruned".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::Pruned)
    );
    assert!("invalid".parse::<WorktreeStatus>().is_err());
}

#[test]
// TODO(TEST003): Bad test function name 'worktree_status_from_str_case_insensitive'. Use 'test_worktree_status_from_str_case_insensitive'.
fn worktree_status_from_str_case_insensitive() {
    assert_eq!(
        "ACTIVE".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::Active)
    );
    assert_eq!(
        "In_Use".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::InUse)
    );
    assert_eq!(
        "PRUNED".parse::<WorktreeStatus>(),
        Ok(WorktreeStatus::Pruned)
    );
}

#[test]
// TODO(TEST003): Bad test function name 'worktree_construction'. Use 'test_worktree_construction'.
fn worktree_construction() {
    let wt = Worktree {
        id: "wt-001".to_string(),
        repository_id: "repo-001".to_string(),
        branch_id: "br-001".to_string(),
        path: "/home/dev/mcb-wt-feat".to_string(),
        status: WorktreeStatus::Active,
        assigned_agent_id: None,
        created_at: 1000,
        updated_at: 1000,
    };
    assert_eq!(wt.id, "wt-001");
    assert_eq!(wt.repository_id, "repo-001");
    assert_eq!(wt.branch_id, "br-001");
    assert_eq!(wt.status, WorktreeStatus::Active);
    assert!(wt.assigned_agent_id.is_none());
}

#[test]
// TODO(TEST003): Bad test function name 'worktree_with_assigned_agent'. Use 'test_worktree_with_assigned_agent'.
fn worktree_with_assigned_agent() {
    let wt = Worktree {
        id: "wt-002".to_string(),
        repository_id: "repo-001".to_string(),
        branch_id: "br-002".to_string(),
        path: "/home/dev/mcb-wt-fix".to_string(),
        status: WorktreeStatus::InUse,
        assigned_agent_id: Some("agent-session-001".to_string()),
        created_at: 2000,
        updated_at: 3000,
    };
    assert_eq!(wt.status, WorktreeStatus::InUse);
    assert_eq!(wt.assigned_agent_id, Some("agent-session-001".to_string()));
}

#[test]
// TODO(TEST003): Bad test function name 'worktree_serialization_roundtrip'. Use 'test_worktree_serialization_roundtrip'.
fn worktree_serialization_roundtrip() {
    let wt = Worktree {
        id: "wt-003".to_string(),
        repository_id: "repo-002".to_string(),
        branch_id: "br-003".to_string(),
        path: "/tmp/worktrees/wt-003".to_string(),
        status: WorktreeStatus::Pruned,
        assigned_agent_id: None,
        created_at: 4000,
        updated_at: 5000,
    };
    let json = serde_json::to_string(&wt).expect("serialize");
    let deserialized: Worktree = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "wt-003");
    assert_eq!(deserialized.status, WorktreeStatus::Pruned);
    assert_eq!(deserialized.path, "/tmp/worktrees/wt-003");
}

#[test]
// TODO(TEST003): Bad test function name 'assignment_construction_active'. Use 'test_assignment_construction_active'.
fn assignment_construction_active() {
    let assign = AgentWorktreeAssignment {
        id: "assign-001".to_string(),
        agent_session_id: "agent-session-001".to_string(),
        worktree_id: "wt-001".to_string(),
        assigned_at: 1000,
        released_at: None,
    };
    assert_eq!(assign.id, "assign-001");
    assert_eq!(assign.agent_session_id, "agent-session-001");
    assert_eq!(assign.worktree_id, "wt-001");
    assert!(assign.released_at.is_none());
}

#[test]
// TODO(TEST003): Bad test function name 'assignment_construction_released'. Use 'test_assignment_construction_released'.
fn assignment_construction_released() {
    let assign = AgentWorktreeAssignment {
        id: "assign-002".to_string(),
        agent_session_id: "agent-session-002".to_string(),
        worktree_id: "wt-002".to_string(),
        assigned_at: 1000,
        released_at: Some(2000),
    };
    assert_eq!(assign.released_at, Some(2000));
}

#[test]
// TODO(TEST003): Bad test function name 'assignment_serialization_roundtrip'. Use 'test_assignment_serialization_roundtrip'.
fn assignment_serialization_roundtrip() {
    let assign = AgentWorktreeAssignment {
        id: "assign-003".to_string(),
        agent_session_id: "agent-session-003".to_string(),
        worktree_id: "wt-003".to_string(),
        assigned_at: 5000,
        released_at: Some(6000),
    };
    let json = serde_json::to_string(&assign).expect("serialize");
    let deserialized: AgentWorktreeAssignment = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "assign-003");
    assert_eq!(deserialized.assigned_at, 5000);
    assert_eq!(deserialized.released_at, Some(6000));
}
