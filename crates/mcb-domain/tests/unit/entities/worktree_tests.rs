use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use rstest::*;

#[rstest]
#[case(WorktreeStatus::Active, "active")]
#[case(WorktreeStatus::InUse, "in_use")]
#[case(WorktreeStatus::Pruned, "pruned")]
fn worktree_status_as_str(#[case] status: WorktreeStatus, #[case] expected: &str) {
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
fn worktree_status_from_str(#[case] input: &str, #[case] expected: Result<WorktreeStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<WorktreeStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<WorktreeStatus>().is_err()),
    }
}

#[rstest]
#[case(
    "wt-001",
    "repo-001",
    "br-001",
    "/home/dev/mcb-wt-feat",
    WorktreeStatus::Active,
    None
)]
#[case(
    "wt-002",
    "repo-001",
    "br-002",
    "/home/dev/mcb-wt-fix",
    WorktreeStatus::InUse,
    Some("agent-session-001")
)]
#[case(
    "wt-003",
    "repo-002",
    "br-003",
    "/tmp/worktrees/wt-003",
    WorktreeStatus::Pruned,
    None
)]
fn worktree_variants(
    #[case] id: &str,
    #[case] repository_id: &str,
    #[case] branch_id: &str,
    #[case] path: &str,
    #[case] status: WorktreeStatus,
    #[case] assigned_agent_id: Option<&str>,
) {
    let wt = Worktree {
        id: id.to_string(),
        repository_id: repository_id.to_string(),
        branch_id: branch_id.to_string(),
        path: path.to_string(),
        status: status.clone(),
        assigned_agent_id: assigned_agent_id.map(str::to_string),
        created_at: 1000,
        updated_at: 1000,
    };

    assert_eq!(wt.id, id);
    assert_eq!(wt.repository_id, repository_id);
    assert_eq!(wt.branch_id, branch_id);
    assert_eq!(wt.path, path);
    assert_eq!(wt.assigned_agent_id.as_deref(), assigned_agent_id);

    let json = serde_json::to_string(&wt).expect("serialize");
    let deserialized: Worktree = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, id);
    assert_eq!(deserialized.status, status);
    assert_eq!(deserialized.path, path);
}

#[rstest]
#[case("assign-001", "agent-session-001", "wt-001", 1000, None)]
#[case("assign-002", "agent-session-002", "wt-002", 1000, Some(2000))]
#[case("assign-003", "agent-session-003", "wt-003", 5000, Some(6000))]
fn assignment_variants(
    #[case] id: &str,
    #[case] agent_session_id: &str,
    #[case] worktree_id: &str,
    #[case] assigned_at: i64,
    #[case] released_at: Option<i64>,
) {
    let assign = AgentWorktreeAssignment {
        id: id.to_string(),
        agent_session_id: agent_session_id.to_string(),
        worktree_id: worktree_id.to_string(),
        assigned_at,
        released_at,
    };
    assert_eq!(assign.id, id);
    assert_eq!(assign.agent_session_id, agent_session_id);
    assert_eq!(assign.worktree_id, worktree_id);
    assert_eq!(assign.released_at, released_at);

    let json = serde_json::to_string(&assign).expect("serialize");
    let deserialized: AgentWorktreeAssignment = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, id);
    assert_eq!(deserialized.assigned_at, assigned_at);
    assert_eq!(deserialized.released_at, released_at);
}
