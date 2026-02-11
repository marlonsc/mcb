//! Tests for project entity (REF003: dedicated test file).

use mcb_domain::entities::project::{
    DependencyType, DetectedProject, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};

#[test]
fn test_project_type_cargo() {
    let pt = ProjectType::Cargo {
        name: "foo".to_string(),
        version: "0.1.0".to_string(),
        dependencies: vec![],
    };
    match &pt {
        ProjectType::Cargo { name, version, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(version, "0.1.0");
        }
        _ => panic!("expected Cargo"),
    }
}

#[test]
fn test_detected_project_has_path_and_id() {
    let p = DetectedProject {
        id: "proj-1".to_string(),
        path: "crates/foo".to_string(),
        project_type: ProjectType::Cargo {
            name: "foo".to_string(),
            version: "0.1.0".to_string(),
            dependencies: vec![],
        },
        parent_repo_id: None,
    };
    assert_eq!(p.id, "proj-1");
    assert_eq!(p.path, "crates/foo");
}

#[test]
fn test_phase_status_from_str() {
    assert_eq!("planned".parse::<PhaseStatus>(), Ok(PhaseStatus::Planned));
    assert_eq!(
        "in_progress".parse::<PhaseStatus>(),
        Ok(PhaseStatus::InProgress)
    );
    assert_eq!("blocked".parse::<PhaseStatus>(), Ok(PhaseStatus::Blocked));
    assert_eq!(
        "completed".parse::<PhaseStatus>(),
        Ok(PhaseStatus::Completed)
    );
    assert_eq!("skipped".parse::<PhaseStatus>(), Ok(PhaseStatus::Skipped));
    assert!("invalid".parse::<PhaseStatus>().is_err());
}

#[test]
fn test_phase_status_as_str() {
    assert_eq!(PhaseStatus::Planned.as_str(), "planned");
    assert_eq!(PhaseStatus::InProgress.as_str(), "in_progress");
    assert_eq!(PhaseStatus::Blocked.as_str(), "blocked");
    assert_eq!(PhaseStatus::Completed.as_str(), "completed");
    assert_eq!(PhaseStatus::Skipped.as_str(), "skipped");
}

#[test]
fn test_issue_type_from_str() {
    assert_eq!("task".parse::<IssueType>(), Ok(IssueType::Task));
    assert_eq!("bug".parse::<IssueType>(), Ok(IssueType::Bug));
    assert_eq!("feature".parse::<IssueType>(), Ok(IssueType::Feature));
    assert_eq!(
        "enhancement".parse::<IssueType>(),
        Ok(IssueType::Enhancement)
    );
    assert_eq!(
        "documentation".parse::<IssueType>(),
        Ok(IssueType::Documentation)
    );
    assert!("invalid".parse::<IssueType>().is_err());
}

#[test]
fn test_issue_status_from_str() {
    assert_eq!("open".parse::<IssueStatus>(), Ok(IssueStatus::Open));
    assert_eq!(
        "in_progress".parse::<IssueStatus>(),
        Ok(IssueStatus::InProgress)
    );
    assert_eq!("blocked".parse::<IssueStatus>(), Ok(IssueStatus::Blocked));
    assert_eq!("resolved".parse::<IssueStatus>(), Ok(IssueStatus::Resolved));
    assert_eq!("closed".parse::<IssueStatus>(), Ok(IssueStatus::Closed));
    assert!("invalid".parse::<IssueStatus>().is_err());
}

#[test]
fn test_dependency_type_from_str() {
    assert_eq!(
        "blocks".parse::<DependencyType>(),
        Ok(DependencyType::Blocks)
    );
    assert_eq!(
        "relates_to".parse::<DependencyType>(),
        Ok(DependencyType::RelatesTo)
    );
    assert_eq!(
        "duplicate_of".parse::<DependencyType>(),
        Ok(DependencyType::DuplicateOf)
    );
    assert_eq!(
        "parent_of".parse::<DependencyType>(),
        Ok(DependencyType::ParentOf)
    );
    assert!("invalid".parse::<DependencyType>().is_err());
}

#[test]
fn test_project_phase_construction() {
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

#[test]
fn test_project_issue_construction() {
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

#[test]
fn test_project_dependency_construction() {
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

#[test]
fn test_project_decision_construction() {
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
