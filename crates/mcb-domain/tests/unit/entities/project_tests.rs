use mcb_domain::entities::project::{
    DependencyType, DetectedProject, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectType,
};
use mcb_domain::utils::tests::utils::{create_test_issue, create_test_phase};
use rstest::{fixture, rstest};

#[rstest]
#[case("foo", "0.1.0")]
fn project_type_cargo(#[case] name: &str, #[case] version: &str) {
    let pt = ProjectType::Cargo {
        name: name.to_owned(),
        version: version.to_owned(),
        dependencies: vec![],
    };
    match &pt {
        ProjectType::Cargo {
            name: actual_name,
            version: actual_version,
            ..
        } => {
            assert_eq!(actual_name, name);
            assert_eq!(actual_version, version);
        }
        ProjectType::Npm { .. }
        | ProjectType::Python { .. }
        | ProjectType::Go { .. }
        | ProjectType::Maven { .. } => panic!("expected Cargo"),
    }
}

#[rstest]
#[case("proj-1", "crates/foo")]
fn detected_project_has_path_and_id(#[case] id: &str, #[case] path: &str) {
    let p = DetectedProject {
        id: id.to_owned(),
        path: path.to_owned(),
        project_type: ProjectType::Cargo {
            name: "foo".to_owned(),
            version: "0.1.0".to_owned(),
            dependencies: vec![],
        },
        parent_repo_id: None,
    };
    assert_eq!(p.id, id);
    assert_eq!(p.path, path);
}

#[rstest]
#[case("planned", Ok(PhaseStatus::Planned))]
#[case("in_progress", Ok(PhaseStatus::InProgress))]
#[case("blocked", Ok(PhaseStatus::Blocked))]
#[case("completed", Ok(PhaseStatus::Completed))]
#[case("skipped", Ok(PhaseStatus::Skipped))]
#[case("invalid", Err(()))]
fn phase_status_from_str(#[case] input: &str, #[case] expected: Result<PhaseStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<PhaseStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<PhaseStatus>().is_err()),
    }
}

#[rstest]
#[case(PhaseStatus::Planned, "planned")]
#[case(PhaseStatus::InProgress, "in_progress")]
#[case(PhaseStatus::Blocked, "blocked")]
#[case(PhaseStatus::Completed, "completed")]
#[case(PhaseStatus::Skipped, "skipped")]
fn phase_status_as_str(#[case] status: PhaseStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case("task", Ok(IssueType::Task))]
#[case("bug", Ok(IssueType::Bug))]
#[case("feature", Ok(IssueType::Feature))]
#[case("enhancement", Ok(IssueType::Enhancement))]
#[case("documentation", Ok(IssueType::Documentation))]
#[case("invalid", Err(()))]
fn issue_type_from_str(#[case] input: &str, #[case] expected: Result<IssueType, ()>) {
    match expected {
        Ok(issue_type) => assert_eq!(input.parse::<IssueType>(), Ok(issue_type)),
        Err(()) => assert!(input.parse::<IssueType>().is_err()),
    }
}

#[rstest]
#[case("open", Ok(IssueStatus::Open))]
#[case("in_progress", Ok(IssueStatus::InProgress))]
#[case("blocked", Ok(IssueStatus::Blocked))]
#[case("resolved", Ok(IssueStatus::Resolved))]
#[case("closed", Ok(IssueStatus::Closed))]
#[case("invalid", Err(()))]
fn issue_status_from_str(#[case] input: &str, #[case] expected: Result<IssueStatus, ()>) {
    match expected {
        Ok(issue_status) => assert_eq!(input.parse::<IssueStatus>(), Ok(issue_status)),
        Err(()) => assert!(input.parse::<IssueStatus>().is_err()),
    }
}

#[rstest]
#[case("blocks", Ok(DependencyType::Blocks))]
#[case("relates_to", Ok(DependencyType::RelatesTo))]
#[case("duplicate_of", Ok(DependencyType::DuplicateOf))]
#[case("parent_of", Ok(DependencyType::ParentOf))]
#[case("invalid", Err(()))]
fn dependency_type_from_str(#[case] input: &str, #[case] expected: Result<DependencyType, ()>) {
    match expected {
        Ok(dep_type) => assert_eq!(input.parse::<DependencyType>(), Ok(dep_type)),
        Err(()) => assert!(input.parse::<DependencyType>().is_err()),
    }
}

use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::entities::project::ProjectPhase;

#[fixture]
fn project_phase() -> ProjectPhase {
    let mut phase = create_test_phase("ph-001", "proj-1");
    phase.sequence = 1;
    phase.status = PhaseStatus::InProgress;
    phase
}

#[rstest]
fn test_project_phase_construction(project_phase: ProjectPhase) {
    assert_eq!(project_phase.id, "ph-001");
    assert_eq!(project_phase.sequence, 1);
    assert_eq!(project_phase.status, PhaseStatus::InProgress);
}

#[fixture]
fn project_issue() -> ProjectIssue {
    let mut issue = create_test_issue("iss-001", "proj-1");
    issue.phase_id = Some("ph-001".to_owned());
    issue.title = "Fix bug".to_owned();
    issue.description = "Something is broken".to_owned();
    issue.issue_type = IssueType::Bug;
    issue.status = IssueStatus::Open;
    issue.priority = 1;
    issue.assignee = Some("alice".to_owned());
    issue.labels = vec!["urgent".to_owned()];
    issue
}

#[rstest]
fn test_project_issue_construction(project_issue: ProjectIssue) {
    assert_eq!(project_issue.id, "iss-001");
    assert_eq!(project_issue.priority, 1);
    assert_eq!(project_issue.issue_type, IssueType::Bug);
}

#[fixture]
fn project_dependency() -> ProjectDependency {
    ProjectDependency {
        id: "dep-001".to_owned(),
        from_issue_id: "iss-002".to_owned(),
        to_issue_id: "iss-001".to_owned(),
        dependency_type: DependencyType::Blocks,
        created_at: 1000,
    }
}

#[rstest]
fn test_project_dependency_construction(project_dependency: ProjectDependency) {
    assert_eq!(project_dependency.id, "dep-001");
    assert_eq!(project_dependency.dependency_type, DependencyType::Blocks);
}

#[fixture]
fn project_decision() -> ProjectDecision {
    ProjectDecision {
        id: "dec-001".to_owned(),
        project_id: "proj-1".to_owned(),
        issue_id: None,
        title: "Use Rust".to_owned(),
        context: "Need a fast language".to_owned(),
        decision: "Rust for performance".to_owned(),
        consequences: "Steeper learning curve".to_owned(),
        created_at: 1000,
    }
}

#[rstest]
fn test_project_decision_construction(project_decision: ProjectDecision) {
    assert_eq!(project_decision.id, "dec-001");
    assert_eq!(project_decision.title, "Use Rust");
}
