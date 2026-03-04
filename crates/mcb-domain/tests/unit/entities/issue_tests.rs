//! Tests for issue entities and enhanced issue fields.

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use rstest::{fixture, rstest};

#[fixture]
fn default_issue() -> ProjectIssue {
    ProjectIssue {
        id: "iss-001".to_owned(),
        org_id: "org-1".to_owned(),
        project_id: "proj-1".to_owned(),
        created_by: "user-1".to_owned(),
        phase_id: Some("phase-1".to_owned()),
        title: "Improve parser".to_owned(),
        description: "Add validation and mapping".to_owned(),
        issue_type: IssueType::Enhancement,
        status: IssueStatus::InProgress,
        priority: 2,
        assignee: Some("user-2".to_owned()),
        labels: vec!["backend".to_owned(), "high-priority".to_owned()],
        estimated_minutes: Some(90),
        actual_minutes: Some(30),
        notes: "Initial implementation started".to_owned(),
        design: "Follow plan_entity layering".to_owned(),
        parent_issue_id: Some("iss-000".to_owned()),
        created_at: 100,
        updated_at: 120,
        closed_at: None,
        closed_reason: String::new(),
    }
}

#[rstest]
fn test_enhanced_project_issue_construction(default_issue: ProjectIssue) {
    assert_eq!(default_issue.org_id, "org-1");
    assert_eq!(default_issue.created_by, "user-1");
    assert_eq!(default_issue.estimated_minutes, Some(90));
    assert_eq!(default_issue.actual_minutes, Some(30));
    assert_eq!(default_issue.parent_issue_id.as_deref(), Some("iss-000"));
    assert_eq!(default_issue.notes, "Initial implementation started");
    assert_eq!(default_issue.design, "Follow plan_entity layering");
}

#[fixture]
fn default_comment() -> IssueComment {
    IssueComment {
        id: "c-1".to_owned(),
        issue_id: "iss-1".to_owned(),
        author_id: "user-1".to_owned(),
        content: "Looks good".to_owned(),
        created_at: 123,
    }
}

#[fixture]
fn default_label() -> IssueLabel {
    IssueLabel {
        id: "l-1".to_owned(),
        org_id: "org-1".to_owned(),
        project_id: "proj-1".to_owned(),
        name: "bug".to_owned(),
        color: "#ff0000".to_owned(),
        created_at: 123,
    }
}

#[rstest]
fn test_issue_comment_serialization_roundtrip(default_comment: IssueComment) {
    let json = serde_json::to_string(&default_comment).expect("serialize issue comment");
    let parsed: IssueComment = serde_json::from_str(&json).expect("deserialize issue comment");

    assert_eq!(parsed.id, "c-1");
    assert_eq!(parsed.content, "Looks good");
}

#[rstest]
fn test_issue_label_serialization_roundtrip(default_label: IssueLabel) {
    let json = serde_json::to_string(&default_label).expect("serialize issue label");
    let parsed: IssueLabel = serde_json::from_str(&json).expect("deserialize issue label");

    assert_eq!(parsed.id, "l-1");
    assert_eq!(parsed.name, "bug");
}

use mcb_domain::value_objects::ids::IssueLabelAssignmentId;
use mcb_utils::utils::id;

#[rstest]
fn test_issue_label_assignment_construction() {
    let id_uuid = id::deterministic("issue_label_assignment", "iss-1:l-1");
    let assignment = IssueLabelAssignment {
        id: IssueLabelAssignmentId::from_uuid(id_uuid),
        issue_id: "iss-1".to_owned(),
        label_id: "l-1".to_owned(),
        created_at: 987,
    };

    assert_eq!(assignment.issue_id, "iss-1");
    assert_eq!(assignment.label_id, "l-1");
    assert_eq!(assignment.created_at, 987);
}
