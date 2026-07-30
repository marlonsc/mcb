use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use rstest::{fixture, rstest};

#[fixture]
fn worktree() -> Worktree {
    Worktree {
        id: "wt-001".to_owned(),
        repository_id: "repo-001".to_owned(),
        branch_id: "br-001".to_owned(),
        path: "/home/dev/mcb-wt-feat".to_owned(),
        status: WorktreeStatus::Active,
        assigned_agent_id: None,
        created_at: 1000,
        updated_at: 1000,
    }
}

#[fixture]
fn assignment() -> AgentWorktreeAssignment {
    AgentWorktreeAssignment {
        id: "assign-001".to_owned(),
        agent_session_id: "agent-session-001".to_owned(),
        worktree_id: "wt-001".to_owned(),
        assigned_at: 1000,
        released_at: None,
    }
}

#[rstest]
fn test_worktree_construction(worktree: Worktree) {
    assert_eq!(worktree.id, "wt-001");
    assert_eq!(worktree.status, WorktreeStatus::Active);
}

#[rstest]
fn test_worktree_serialization_roundtrip(worktree: Worktree) {
    let json = serde_json::to_string(&worktree).expect("serialize");
    let deserialized: Worktree = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, worktree.id);
}

#[rstest]
fn test_assignment_construction(assignment: AgentWorktreeAssignment) {
    assert_eq!(assignment.id, "assign-001");
    assert!(assignment.released_at.is_none());
}

#[rstest]
fn test_assignment_serialization_roundtrip(assignment: AgentWorktreeAssignment) {
    let json = serde_json::to_string(&assignment).expect("serialize");
    let deserialized: AgentWorktreeAssignment = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, assignment.id);
}

#[rstest]
#[case(WorktreeStatus::Active, "active")]
#[case(WorktreeStatus::InUse, "in_use")]
#[case(WorktreeStatus::Pruned, "pruned")]
fn test_worktree_status_as_str(#[case] status: WorktreeStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case("active", Ok(WorktreeStatus::Active))]
#[case("in_use", Ok(WorktreeStatus::InUse))]
#[case("pruned", Ok(WorktreeStatus::Pruned))]
#[case("ACTIVE", Ok(WorktreeStatus::Active))]
#[case("In_Use", Ok(WorktreeStatus::InUse))]
#[case("PRUNED", Ok(WorktreeStatus::Pruned))]
#[case("invalid", Err(()))]
fn test_worktree_status_from_str(
    #[case] input: &str,
    #[case] expected: Result<WorktreeStatus, ()>,
) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<WorktreeStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<WorktreeStatus>().is_err()),
    }
}
