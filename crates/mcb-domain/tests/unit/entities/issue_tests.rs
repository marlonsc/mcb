//! Tests for issue entities and enhanced issue fields.

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use rstest::rstest;

#[rstest]
fn test_enhanced_project_issue_construction() {
    let issue = ProjectIssue {
        id: "iss-001".to_string(),
        org_id: "org-1".to_string(),
        project_id: "proj-1".to_string(),
        created_by: "user-1".to_string(),
        phase_id: Some("phase-1".to_string()),
        title: "Improve parser".to_string(),
        description: "Add validation and mapping".to_string(),
        issue_type: IssueType::Enhancement,
        status: IssueStatus::InProgress,
        priority: 2,
        assignee: Some("user-2".to_string()),
        labels: vec!["backend".to_string(), "high-priority".to_string()],
        estimated_minutes: Some(90),
        actual_minutes: Some(30),
        notes: "Initial implementation started".to_string(),
        design: "Follow plan_entity layering".to_string(),
        parent_issue_id: Some("iss-000".to_string()),
        created_at: 100,
        updated_at: 120,
        closed_at: None,
        closed_reason: String::new(),
    };

    assert_eq!(issue.org_id, "org-1");
    assert_eq!(issue.created_by, "user-1");
    assert_eq!(issue.estimated_minutes, Some(90));
    assert_eq!(issue.actual_minutes, Some(30));
    assert_eq!(issue.parent_issue_id.as_deref(), Some("iss-000"));
    assert_eq!(issue.notes, "Initial implementation started");
    assert_eq!(issue.design, "Follow plan_entity layering");
}

#[rstest]
fn test_issue_comment_serialization_roundtrip() {
    let comment = IssueComment {
        id: "c-1".to_string(),
        issue_id: "iss-1".to_string(),
        author_id: "user-1".to_string(),
        content: "Looks good".to_string(),
        created_at: 123,
    };

    let json = serde_json::to_string(&comment).expect("serialize issue comment");
    let parsed: IssueComment = serde_json::from_str(&json).expect("deserialize issue comment");

    assert_eq!(parsed.id, "c-1");
    assert_eq!(parsed.issue_id, "iss-1");
    assert_eq!(parsed.author_id, "user-1");
    assert_eq!(parsed.content, "Looks good");
    assert_eq!(parsed.created_at, 123);
}

#[rstest]
fn test_issue_label_serialization_roundtrip() {
    let label = IssueLabel {
        id: "l-1".to_string(),
        org_id: "org-1".to_string(),
        project_id: "proj-1".to_string(),
        name: "bug".to_string(),
        color: "#ff0000".to_string(),
        created_at: 123,
    };

    let json = serde_json::to_string(&label).expect("serialize issue label");
    let parsed: IssueLabel = serde_json::from_str(&json).expect("deserialize issue label");

    assert_eq!(parsed.id, "l-1");
    assert_eq!(parsed.org_id, "org-1");
    assert_eq!(parsed.project_id, "proj-1");
    assert_eq!(parsed.name, "bug");
    assert_eq!(parsed.color, "#ff0000");
    assert_eq!(parsed.created_at, 123);
}

use mcb_domain::utils::id;
use mcb_domain::value_objects::ids::IssueLabelAssignmentId;

#[rstest]
fn test_issue_label_assignment_construction() {
    let id_uuid = id::deterministic("issue_label_assignment", "iss-1:l-1");
    let assignment = IssueLabelAssignment {
        id: IssueLabelAssignmentId::from_uuid(id_uuid),
        issue_id: "iss-1".to_string(),
        label_id: "l-1".to_string(),
        created_at: 987,
    };

    assert_eq!(assignment.issue_id, "iss-1");
    assert_eq!(assignment.label_id, "l-1");
    assert_eq!(assignment.created_at, 987);
}
