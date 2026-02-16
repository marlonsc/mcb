use rstest::rstest;
use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::issue_entity_repository::{
    IssueCommentRegistry, IssueLabelAssignmentManager, IssueLabelRegistry, IssueRegistry,
};
use mcb_providers::database::SqliteIssueEntityRepository;

use crate::common::entity_test_utils::{
    TEST_NOW, assert_not_found, seed_default_scope, seed_isolated_org_scope, setup_executor,
};

async fn setup_repo() -> (
    SqliteIssueEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let (executor, temp_dir) = setup_executor().await;
    seed_default_scope(executor.as_ref()).await;
    let repo = SqliteIssueEntityRepository::new(Arc::clone(&executor));
    (repo, executor, temp_dir)
}

fn create_test_issue(id: &str) -> ProjectIssue {
    ProjectIssue {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        project_id: "proj-1".to_owned(),
        created_by: "user-1".to_owned(),
        phase_id: None,
        title: format!("Issue {id}"),
        description: format!("Description for {id}"),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        estimated_minutes: None,
        actual_minutes: None,
        notes: String::new(),
        design: String::new(),
        parent_issue_id: None,
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
        closed_at: None,
        closed_reason: String::new(),
    }
}

fn create_test_comment(id: &str, issue_id: &str) -> IssueComment {
    IssueComment {
        id: id.to_owned(),
        issue_id: issue_id.to_owned(),
        author_id: "user-1".to_owned(),
        content: format!("Comment {id}"),
        created_at: TEST_NOW,
    }
}

fn create_test_label(id: &str, name: &str, color: &str) -> IssueLabel {
    IssueLabel {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        project_id: "proj-1".to_owned(),
        name: name.to_owned(),
        color: color.to_owned(),
        created_at: TEST_NOW,
    }
}

#[tokio::test]
async fn test_issue_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let issue = create_test_issue("issue-1");

    repo.create_issue(&issue).await.expect("create");

    let retrieved = repo
        .get_issue(DEFAULT_ORG_ID, "issue-1")
        .await
        .expect("get");
    assert_eq!(retrieved.title, "Issue issue-1");
    assert_eq!(retrieved.status, IssueStatus::Open);

    let list = repo
        .list_issues(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = issue.clone();
    updated.status = IssueStatus::InProgress;
    updated.updated_at = 2_000_000;
    repo.update_issue(&updated).await.expect("update");

    let after_update = repo
        .get_issue(DEFAULT_ORG_ID, "issue-1")
        .await
        .expect("get");
    assert_eq!(after_update.status, IssueStatus::InProgress);

    repo.delete_issue(DEFAULT_ORG_ID, "issue-1")
        .await
        .expect("delete");
    assert_not_found(repo.get_issue(DEFAULT_ORG_ID, "issue-1").await);
}

#[tokio::test]
async fn test_comment_lifecycle() {
    let (repo, _executor, _temp) = setup_repo().await;
    let issue = create_test_issue("issue-1");
    repo.create_issue(&issue).await.expect("create issue");

    let c1 = create_test_comment("c1", "issue-1");
    let c2 = create_test_comment("c2", "issue-1");
    repo.create_comment(&c1).await.expect("create c1");
    repo.create_comment(&c2).await.expect("create c2");

    let retrieved = repo.get_comment("c1").await.expect("get");
    assert_eq!(retrieved.content, "Comment c1");

    let comments = repo.list_comments_by_issue("issue-1").await.expect("list");
    assert_eq!(comments.len(), 2);

    repo.delete_comment("c1").await.expect("delete c1");
    let after_delete = repo.list_comments_by_issue("issue-1").await.expect("list");
    assert_eq!(after_delete.len(), 1);
    assert_eq!(after_delete[0].id, "c2");
}

#[tokio::test]
async fn test_label_lifecycle() {
    let (repo, _executor, _temp) = setup_repo().await;

    let l1 = create_test_label("lbl-1", "bug", "#ff0000");
    let l2 = create_test_label("lbl-2", "feature", "#00ff00");
    repo.create_label(&l1).await.expect("create l1");
    repo.create_label(&l2).await.expect("create l2");

    let retrieved = repo.get_label("lbl-1").await.expect("get");
    assert_eq!(retrieved.name, "bug");
    assert_eq!(retrieved.color, "#ff0000");

    let labels = repo
        .list_labels(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("list");
    assert_eq!(labels.len(), 2);

    repo.delete_label("lbl-1").await.expect("delete");
    let after_delete = repo
        .list_labels(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("list");
    assert_eq!(after_delete.len(), 1);
}

#[tokio::test]
async fn test_label_assignment() {
    let (repo, _executor, _temp) = setup_repo().await;
    let issue = create_test_issue("issue-1");
    repo.create_issue(&issue).await.expect("create issue");

    let l1 = create_test_label("lbl-1", "bug", "#ff0000");
    let l2 = create_test_label("lbl-2", "urgent", "#ff8800");
    repo.create_label(&l1).await.expect("create l1");
    repo.create_label(&l2).await.expect("create l2");

    use mcb_domain::utils::id;
    use mcb_domain::value_objects::ids::IssueLabelAssignmentId;

    let a1 = IssueLabelAssignment {
        id: IssueLabelAssignmentId::from_uuid(id::deterministic(
            "issue_label_assignment",
            "issue-1:lbl-1",
        )),
        issue_id: "issue-1".to_owned(),
        label_id: "lbl-1".to_owned(),
        created_at: TEST_NOW,
    };
    let a2 = IssueLabelAssignment {
        id: IssueLabelAssignmentId::from_uuid(id::deterministic(
            "issue_label_assignment",
            "issue-1:lbl-2",
        )),
        issue_id: "issue-1".to_owned(),
        label_id: "lbl-2".to_owned(),
        created_at: TEST_NOW,
    };
    repo.assign_label(&a1).await.expect("assign l1");
    repo.assign_label(&a2).await.expect("assign l2");

    let assigned = repo
        .list_labels_for_issue("issue-1")
        .await
        .expect("list labels for issue");
    assert_eq!(assigned.len(), 2);

    repo.unassign_label("issue-1", "lbl-1")
        .await
        .expect("unassign l1");
    let after_unassign = repo.list_labels_for_issue("issue-1").await.expect("list");
    assert_eq!(after_unassign.len(), 1);
    assert_eq!(after_unassign[0].id, "lbl-2");
}

#[rstest]
#[case("org-A", true)]
#[case("org-B", false)]
#[tokio::test]
async fn org_isolation_issues(#[case] org_id: &str, #[case] should_find: bool) {
    let (executor, _temp_dir) = setup_executor().await;

    for org_id in &["org-A", "org-B"] {
        seed_isolated_org_scope(executor.as_ref(), org_id).await;
    }

    let repo = SqliteIssueEntityRepository::new(executor);
    let issue = ProjectIssue {
        id: "issue-iso".to_owned(),
        org_id: "org-A".to_owned(),
        project_id: "proj-org-A".to_owned(),
        created_by: "user-org-A".to_owned(),
        phase_id: None,
        title: "Org A Issue".to_owned(),
        description: "belongs to A".to_owned(),
        issue_type: IssueType::Bug,
        status: IssueStatus::Open,
        priority: 1,
        assignee: None,
        labels: vec![],
        estimated_minutes: None,
        actual_minutes: None,
        notes: String::new(),
        design: String::new(),
        parent_issue_id: None,
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
        closed_at: None,
        closed_reason: String::new(),
    };
    repo.create_issue(&issue).await.expect("create");

    let get_result = repo.get_issue(org_id, "issue-iso").await;
    if should_find {
        assert!(get_result.is_ok());
    } else {
        assert_not_found(get_result);
        assert!(
            repo.list_issues("org-B", "proj-org-B")
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[tokio::test]
async fn test_delete_issue_with_comments_fails() {
    let (repo, _executor, _temp) = setup_repo().await;
    let issue = create_test_issue("issue-fk");
    repo.create_issue(&issue).await.expect("create issue");

    let comment = create_test_comment("c1", "issue-fk");
    repo.create_comment(&comment).await.expect("create comment");

    let result = repo.delete_issue(DEFAULT_ORG_ID, "issue-fk").await;
    assert!(
        result.is_err(),
        "Deleting an issue with comments should fail due to FK constraint"
    );
}
