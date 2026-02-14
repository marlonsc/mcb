//! Tests for project entity (REF003: dedicated test file).

use mcb_domain::entities::project::{
    DependencyType, DetectedProject, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};
use rstest::*;

#[rstest]
#[case("foo", "0.1.0")]
fn project_type_cargo(#[case] name: &str, #[case] version: &str) {
    let pt = ProjectType::Cargo {
        name: name.to_string(),
        version: version.to_string(),
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
        _ => panic!("expected Cargo"),
    }
}

#[rstest]
#[case("proj-1", "crates/foo")]
fn detected_project_has_path_and_id(#[case] id: &str, #[case] path: &str) {
    let p = DetectedProject {
        id: id.to_string(),
        path: path.to_string(),
        project_type: ProjectType::Cargo {
            name: "foo".to_string(),
            version: "0.1.0".to_string(),
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

#[rstest]
#[case("phase")]
#[case("issue")]
#[case("dependency")]
#[case("decision")]
fn project_entities_construction(#[case] entity: &str) {
    match entity {
        "phase" => {
            let phase = ProjectPhase {
                id: "ph-001".to_string(),
                project_id: "proj-1".to_string(),
                name: "Phase 1".to_string(),
                description: "Initial setup".to_string(),
                sequence: 1,
                status: PhaseStatus::InProgress,
                started_at: Some(1000),
                completed_at: None,
                created_at: 900,
                updated_at: 1000,
            };
            assert_eq!(phase.id, "ph-001");
            assert_eq!(phase.sequence, 1);
            assert_eq!(phase.status, PhaseStatus::InProgress);
        }
        "issue" => {
            let issue = ProjectIssue {
                id: "iss-001".to_string(),
                org_id: "org-1".to_string(),
                project_id: "proj-1".to_string(),
                created_by: "creator".to_string(),
                phase_id: Some("ph-001".to_string()),
                title: "Fix bug".to_string(),
                description: "Something is broken".to_string(),
                issue_type: IssueType::Bug,
                status: IssueStatus::Open,
                priority: 1,
                assignee: Some("alice".to_string()),
                labels: vec!["urgent".to_string()],
                estimated_minutes: None,
                actual_minutes: None,
                notes: String::new(),
                design: String::new(),
                parent_issue_id: None,
                created_at: 1000,
                updated_at: 1000,
                closed_at: None,
                closed_reason: String::new(),
            };
            assert_eq!(issue.id, "iss-001");
            assert_eq!(issue.priority, 1);
            assert_eq!(issue.issue_type, IssueType::Bug);
        }
        "dependency" => {
            let dep = ProjectDependency {
                id: "dep-001".to_string(),
                from_issue_id: "iss-002".to_string(),
                to_issue_id: "iss-001".to_string(),
                dependency_type: DependencyType::Blocks,
                created_at: 1000,
            };
            assert_eq!(dep.id, "dep-001");
            assert_eq!(dep.dependency_type, DependencyType::Blocks);
        }
        "decision" => {
            let decision = ProjectDecision {
                id: "dec-001".to_string(),
                project_id: "proj-1".to_string(),
                issue_id: None,
                title: "Use Rust".to_string(),
                context: "Need a fast language".to_string(),
                decision: "Rust for performance".to_string(),
                consequences: "Steeper learning curve".to_string(),
                created_at: 1000,
            };
            assert_eq!(decision.id, "dec-001");
            assert_eq!(decision.title, "Use Rust");
        }
        _ => unreachable!(),
    }
}
