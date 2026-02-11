use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::IssueEntityRepository;
use mcb_providers::database::{
    SqliteIssueEntityRepository, create_memory_repository_with_executor,
};

async fn setup_repo() -> (
    SqliteIssueEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");
    seed_prerequisites(executor.as_ref()).await;
    let repo = SqliteIssueEntityRepository::new(Arc::clone(&executor));
    (repo, executor, temp_dir)
}

async fn seed_prerequisites(executor: &dyn DatabaseExecutor) {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("{}".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed org");
    executor
        .execute(
            "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("proj-1".to_string()),
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("Test Project".to_string()),
                SqlParam::String("/test".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed project");
    executor
        .execute(
            "INSERT INTO users (id, org_id, email, display_name, role, api_key_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("user-1".to_string()),
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("test@example.com".to_string()),
                SqlParam::String("Test User".to_string()),
                SqlParam::String("admin".to_string()),
                SqlParam::Null,
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed user");
}

fn create_test_issue(id: &str) -> ProjectIssue {
    let now = 1_000_000_i64;
    ProjectIssue {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        project_id: "proj-1".to_string(),
        created_by: "user-1".to_string(),
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
        created_at: now,
        updated_at: now,
        closed_at: None,
        closed_reason: String::new(),
    }
}

fn create_test_comment(id: &str, issue_id: &str) -> IssueComment {
    let now = 1_000_000_i64;
    IssueComment {
        id: id.to_string(),
        issue_id: issue_id.to_string(),
        author_id: "user-1".to_string(),
        content: format!("Comment {id}"),
        created_at: now,
    }
}

fn create_test_label(id: &str, name: &str, color: &str) -> IssueLabel {
    let now = 1_000_000_i64;
    IssueLabel {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        project_id: "proj-1".to_string(),
        name: name.to_string(),
        color: color.to_string(),
        created_at: now,
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
    assert!(repo.get_issue(DEFAULT_ORG_ID, "issue-1").await.is_err());
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

    let now = 1_000_000_i64;
    let a1 = IssueLabelAssignment {
        issue_id: "issue-1".to_string(),
        label_id: "lbl-1".to_string(),
        created_at: now,
    };
    let a2 = IssueLabelAssignment {
        issue_id: "issue-1".to_string(),
        label_id: "lbl-2".to_string(),
        created_at: now,
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

#[tokio::test]
async fn test_org_isolation_issues() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");

    for org_id in &["org-A", "org-B"] {
        executor
            .execute(
                "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String("{}".to_string()),
                    SqlParam::I64(0),
                    SqlParam::I64(0),
                ],
            )
            .await
            .expect("seed org");
        executor
            .execute(
                "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(format!("proj-{org_id}")),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(format!("Project {org_id}")),
                    SqlParam::String(format!("/{org_id}")),
                    SqlParam::I64(0),
                    SqlParam::I64(0),
                ],
            )
            .await
            .expect("seed project");
        executor
            .execute(
                "INSERT INTO users (id, org_id, email, display_name, role, api_key_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(format!("user-{org_id}")),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(format!("{org_id}@test.com")),
                    SqlParam::String(format!("User {org_id}")),
                    SqlParam::String("admin".to_string()),
                    SqlParam::Null,
                    SqlParam::I64(0),
                    SqlParam::I64(0),
                ],
            )
            .await
            .expect("seed user");
    }

    let repo = SqliteIssueEntityRepository::new(executor);
    let now = 1_000_000_i64;
    let issue = ProjectIssue {
        id: "issue-iso".to_string(),
        org_id: "org-A".to_string(),
        project_id: "proj-org-A".to_string(),
        created_by: "user-org-A".to_string(),
        phase_id: None,
        title: "Org A Issue".to_string(),
        description: "belongs to A".to_string(),
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
        created_at: now,
        updated_at: now,
        closed_at: None,
        closed_reason: String::new(),
    };
    repo.create_issue(&issue).await.expect("create");

    assert!(repo.get_issue("org-A", "issue-iso").await.is_ok());
    assert!(repo.get_issue("org-B", "issue-iso").await.is_err());
    assert!(
        repo.list_issues("org-B", "proj-org-B")
            .await
            .unwrap()
            .is_empty()
    );
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
