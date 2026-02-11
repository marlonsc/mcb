use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::VcsEntityRepository;
use mcb_providers::database::{SqliteVcsEntityRepository, create_memory_repository_with_executor};

async fn setup_repo() -> (
    SqliteVcsEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");
    seed_org_and_project(executor.as_ref()).await;
    let repo = SqliteVcsEntityRepository::new(Arc::clone(&executor));
    (repo, executor, temp_dir)
}

async fn seed_org_and_project(executor: &dyn DatabaseExecutor) {
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
}

fn create_test_repository(id: &str, project_id: &str) -> Repository {
    let now = 1_000_000_i64;
    Repository {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        project_id: project_id.to_string(),
        name: format!("repo-{id}"),
        url: format!("https://example.com/{id}.git"),
        local_path: format!("/tmp/{id}"),
        vcs_type: VcsType::Git,
        created_at: now,
        updated_at: now,
    }
}

fn create_test_branch(id: &str, repository_id: &str, name: &str) -> Branch {
    let now = 1_000_000_i64;
    Branch {
        id: id.to_string(),
        repository_id: repository_id.to_string(),
        name: name.to_string(),
        is_default: name == "main",
        head_commit: "abc123".to_string(),
        upstream: None,
        created_at: now,
    }
}

fn create_test_worktree(id: &str, repository_id: &str, branch_id: &str) -> Worktree {
    let now = 1_000_000_i64;
    Worktree {
        id: id.to_string(),
        repository_id: repository_id.to_string(),
        branch_id: branch_id.to_string(),
        path: format!("/tmp/worktree-{id}"),
        status: WorktreeStatus::Active,
        assigned_agent_id: None,
        created_at: now,
        updated_at: now,
    }
}

async fn seed_agent_session(executor: &dyn DatabaseExecutor) {
    executor
        .execute(
            "INSERT INTO session_summaries (id, project_id, session_id, topics, decisions, next_steps, key_files, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("summ-1".to_string()),
                SqlParam::String("proj-1".to_string()),
                SqlParam::String("sid-1".to_string()),
                SqlParam::String("[]".to_string()),
                SqlParam::String("[]".to_string()),
                SqlParam::String("[]".to_string()),
                SqlParam::String("[]".to_string()),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed summary");
    executor
        .execute(
            "INSERT INTO agent_sessions (id, session_summary_id, agent_type, model, parent_session_id, started_at, ended_at, duration_ms, status, prompt_summary, result_summary, token_count, tool_calls_count, delegations_count, project_id, worktree_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("agent-1".to_string()),
                SqlParam::String("summ-1".to_string()),
                SqlParam::String("sisyphus".to_string()),
                SqlParam::String("test".to_string()),
                SqlParam::Null,
                SqlParam::I64(0),
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::String("active".to_string()),
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
            ],
        )
        .await
        .expect("seed agent session");
}

#[tokio::test]
async fn test_repository_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let vcs_repo = create_test_repository("repo-1", "proj-1");

    repo.create_repository(&vcs_repo).await.expect("create");

    let retrieved = repo
        .get_repository(DEFAULT_ORG_ID, "repo-1")
        .await
        .expect("get");
    let r = retrieved;
    assert_eq!(r.id, "repo-1");
    assert_eq!(r.name, "repo-repo-1");

    let list = repo
        .list_repositories(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = vcs_repo.clone();
    updated.name = "updated-name".to_string();
    updated.updated_at = 2_000_000;
    repo.update_repository(&updated).await.expect("update");

    let after_update = repo
        .get_repository(DEFAULT_ORG_ID, "repo-1")
        .await
        .expect("get after update");
    assert_eq!(after_update.name, "updated-name");

    repo.delete_repository(DEFAULT_ORG_ID, "repo-1")
        .await
        .expect("delete");
    assert!(repo.get_repository(DEFAULT_ORG_ID, "repo-1").await.is_err());
}

#[tokio::test]
async fn test_branch_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let vcs_repo = create_test_repository("repo-1", "proj-1");
    repo.create_repository(&vcs_repo)
        .await
        .expect("create repo");

    let branch = create_test_branch("branch-1", "repo-1", "main");
    repo.create_branch(&branch).await.expect("create branch");

    let retrieved = repo.get_branch("branch-1").await.expect("get");
    assert_eq!(retrieved.name, "main");
    assert!(retrieved.is_default);

    let list = repo.list_branches("repo-1").await.expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = branch.clone();
    updated.head_commit = "def456".to_string();
    repo.update_branch(&updated).await.expect("update");

    let after_update = repo.get_branch("branch-1").await.expect("get");
    assert_eq!(after_update.head_commit, "def456");

    repo.delete_branch("branch-1").await.expect("delete");
    assert!(repo.get_branch("branch-1").await.is_err());
}

#[tokio::test]
async fn test_worktree_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let vcs_repo = create_test_repository("repo-1", "proj-1");
    repo.create_repository(&vcs_repo)
        .await
        .expect("create repo");
    let branch = create_test_branch("branch-1", "repo-1", "main");
    repo.create_branch(&branch).await.expect("create branch");

    let wt = create_test_worktree("wt-1", "repo-1", "branch-1");
    repo.create_worktree(&wt).await.expect("create worktree");

    let retrieved = repo.get_worktree("wt-1").await.expect("get");
    assert_eq!(retrieved.path, "/tmp/worktree-wt-1");

    let list = repo.list_worktrees("repo-1").await.expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = wt.clone();
    updated.status = WorktreeStatus::InUse;
    updated.updated_at = 2_000_000;
    repo.update_worktree(&updated).await.expect("update");

    let after_update = repo.get_worktree("wt-1").await.expect("get");
    assert_eq!(after_update.status, WorktreeStatus::InUse);

    repo.delete_worktree("wt-1").await.expect("delete");
    assert!(repo.get_worktree("wt-1").await.is_err());
}

#[tokio::test]
async fn test_assignment_lifecycle() {
    let (repo, executor, _temp) = setup_repo().await;
    let vcs_repo = create_test_repository("repo-1", "proj-1");
    repo.create_repository(&vcs_repo)
        .await
        .expect("create repo");
    let branch = create_test_branch("branch-1", "repo-1", "main");
    repo.create_branch(&branch).await.expect("create branch");
    let wt = create_test_worktree("wt-1", "repo-1", "branch-1");
    repo.create_worktree(&wt).await.expect("create worktree");
    seed_agent_session(executor.as_ref()).await;

    let assignment = AgentWorktreeAssignment {
        id: "asgn-1".to_string(),
        agent_session_id: "agent-1".to_string(),
        worktree_id: "wt-1".to_string(),
        assigned_at: 1_000_000,
        released_at: None,
    };
    repo.create_assignment(&assignment)
        .await
        .expect("create assignment");

    let retrieved = repo.get_assignment("asgn-1").await.expect("get");
    assert_eq!(retrieved.agent_session_id, "agent-1");
    assert!(retrieved.released_at.is_none());

    let list = repo
        .list_assignments_by_worktree("wt-1")
        .await
        .expect("list");
    assert_eq!(list.len(), 1);

    repo.release_assignment("asgn-1", 2_000_000)
        .await
        .expect("release");

    let after_release = repo.get_assignment("asgn-1").await.expect("get");
    assert_eq!(after_release.released_at, Some(2_000_000));
}

#[tokio::test]
async fn test_org_isolation_repositories() {
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
    }

    let repo = SqliteVcsEntityRepository::new(executor);
    let vcs_repo = Repository {
        id: "repo-iso".to_string(),
        org_id: "org-A".to_string(),
        project_id: "proj-org-A".to_string(),
        name: "Org A Repo".to_string(),
        url: "https://example.com/a.git".to_string(),
        local_path: "/tmp/a".to_string(),
        vcs_type: VcsType::Git,
        created_at: 1_000_000,
        updated_at: 1_000_000,
    };
    repo.create_repository(&vcs_repo).await.expect("create");

    assert!(repo.get_repository("org-A", "repo-iso").await.is_ok());
    assert!(repo.get_repository("org-B", "repo-iso").await.is_err());
    assert!(
        repo.list_repositories("org-B", "proj-org-B")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_list_branches_filters_by_repository() {
    let (repo, _executor, _temp) = setup_repo().await;

    let repo1 = create_test_repository("repo-1", "proj-1");
    let repo2 = create_test_repository("repo-2", "proj-1");
    repo.create_repository(&repo1).await.expect("create repo 1");
    repo.create_repository(&repo2).await.expect("create repo 2");

    let b1 = create_test_branch("b1", "repo-1", "main");
    let b2 = create_test_branch("b2", "repo-2", "develop");
    repo.create_branch(&b1).await.expect("create b1");
    repo.create_branch(&b2).await.expect("create b2");

    let list_1 = repo.list_branches("repo-1").await.expect("list");
    assert_eq!(list_1.len(), 1);
    assert_eq!(list_1[0].name, "main");

    let list_2 = repo.list_branches("repo-2").await.expect("list");
    assert_eq!(list_2.len(), 1);
    assert_eq!(list_2[0].name, "develop");
}

#[tokio::test]
async fn test_list_worktrees_filters_by_repository() {
    let (repo, _executor, _temp) = setup_repo().await;

    let repo1 = create_test_repository("repo-1", "proj-1");
    let repo2 = create_test_repository("repo-2", "proj-1");
    repo.create_repository(&repo1).await.expect("create repo 1");
    repo.create_repository(&repo2).await.expect("create repo 2");

    let b1 = create_test_branch("b1", "repo-1", "main");
    let b2 = create_test_branch("b2", "repo-2", "develop");
    repo.create_branch(&b1).await.expect("create b1");
    repo.create_branch(&b2).await.expect("create b2");

    let wt1 = create_test_worktree("wt-1", "repo-1", "b1");
    let wt2 = create_test_worktree("wt-2", "repo-2", "b2");
    repo.create_worktree(&wt1).await.expect("create wt1");
    repo.create_worktree(&wt2).await.expect("create wt2");

    let list_1 = repo.list_worktrees("repo-1").await.expect("list");
    assert_eq!(list_1.len(), 1);
    assert_eq!(list_1[0].id, "wt-1");

    let list_2 = repo.list_worktrees("repo-2").await.expect("list");
    assert_eq!(list_2.len(), 1);
    assert_eq!(list_2[0].id, "wt-2");
}
