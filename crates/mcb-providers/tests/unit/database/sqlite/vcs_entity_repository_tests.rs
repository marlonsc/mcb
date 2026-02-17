use rstest::rstest;
use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use mcb_domain::ports::{AssignmentManager, BranchRegistry, RepositoryRegistry, WorktreeManager};
use mcb_domain::ports::{DatabaseExecutor, SqlParam};
use mcb_providers::database::SqliteVcsEntityRepository;

use crate::utils::entity::{
    TEST_NOW, TestResult, assert_not_found, seed_default_scope, seed_isolated_org_scope,
    seed_project, setup_executor,
};

async fn setup_repo() -> TestResult<(
    SqliteVcsEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
)> {
    let (executor, temp_dir) = setup_executor().await?;
    seed_default_scope(executor.as_ref()).await?;
    seed_project(
        executor.as_ref(),
        "proj-2",
        DEFAULT_ORG_ID,
        "Test Project 2",
        "/test-2",
    )
    .await?;
    let repo = SqliteVcsEntityRepository::new(Arc::clone(&executor));
    Ok((repo, executor, temp_dir))
}

fn create_test_repository(id: &str, project_id: &str) -> Repository {
    Repository {
        metadata: mcb_domain::entities::EntityMetadata {
            id: id.to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        org_id: DEFAULT_ORG_ID.to_owned(),
        project_id: project_id.to_owned(),
        name: format!("repo-{id}"),
        url: format!("https://example.com/{id}.git"),
        local_path: format!("/tmp/{id}"),
        vcs_type: VcsType::Git,
    }
}

fn create_test_branch(id: &str, repository_id: &str, name: &str) -> Branch {
    Branch {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        repository_id: repository_id.to_owned(),
        name: name.to_owned(),
        is_default: name == "main",
        head_commit: "abc123".to_owned(),
        upstream: None,
        created_at: TEST_NOW,
    }
}

fn create_test_worktree(id: &str, repository_id: &str, branch_id: &str) -> Worktree {
    Worktree {
        metadata: mcb_domain::entities::EntityMetadata {
            id: id.to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        repository_id: repository_id.to_owned(),
        branch_id: branch_id.to_owned(),
        path: format!("/tmp/worktree-{id}"),
        status: WorktreeStatus::Active,
        assigned_agent_id: None,
    }
}

async fn seed_agent_session(executor: &dyn DatabaseExecutor) -> TestResult {
    executor
        .execute(
            "INSERT INTO session_summaries (id, project_id, org_id, session_id, topics, decisions, next_steps, key_files, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("summ-1".to_owned()),
                SqlParam::String("proj-1".to_owned()),
                SqlParam::String(DEFAULT_ORG_ID.to_owned()),
                SqlParam::String("sid-1".to_owned()),
                SqlParam::String("[]".to_owned()),
                SqlParam::String("[]".to_owned()),
                SqlParam::String("[]".to_owned()),
                SqlParam::String("[]".to_owned()),
                SqlParam::I64(0),
            ],
        )
        .await?;
    executor
        .execute(
            "INSERT INTO agent_sessions (id, session_summary_id, agent_type, model, parent_session_id, started_at, ended_at, duration_ms, status, prompt_summary, result_summary, token_count, tool_calls_count, delegations_count, project_id, worktree_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String("agent-1".to_owned()),
                SqlParam::String("summ-1".to_owned()),
                SqlParam::String("sisyphus".to_owned()),
                SqlParam::String("test".to_owned()),
                SqlParam::Null,
                SqlParam::I64(0),
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::String("active".to_owned()),
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
                SqlParam::Null,
            ],
        )
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_repository_crud() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let vcs_repo = create_test_repository("repo-1", "proj-1");

    repo.create_repository(&vcs_repo).await?;

    let retrieved = repo.get_repository(DEFAULT_ORG_ID, "repo-1").await?;
    let r = retrieved;
    assert_eq!(r.metadata.id, "repo-1");
    assert_eq!(r.name, "repo-repo-1");

    let list = repo.list_repositories(DEFAULT_ORG_ID, "proj-1").await?;
    assert_eq!(list.len(), 1);

    let mut updated = vcs_repo.clone();
    updated.name = "updated-name".to_owned();
    updated.metadata.updated_at = 2_000_000;
    repo.update_repository(&updated).await?;

    let after_update = repo.get_repository(DEFAULT_ORG_ID, "repo-1").await?;
    assert_eq!(after_update.name, "updated-name");

    repo.delete_repository(DEFAULT_ORG_ID, "repo-1").await?;
    assert_not_found(&repo.get_repository(DEFAULT_ORG_ID, "repo-1").await);
    Ok(())
}

#[rstest]
#[case("branch")]
#[case("worktree")]
#[tokio::test]
async fn branch_and_worktree_crud(#[case] entity_kind: &str) -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let vcs_repo = create_test_repository("repo-1", "proj-1");
    repo.create_repository(&vcs_repo).await?;

    let branch = create_test_branch("branch-1", "repo-1", "main");
    repo.create_branch(&branch).await?;

    if entity_kind == "branch" {
        let retrieved = repo.get_branch("branch-1").await?;
        assert_eq!(retrieved.name, "main");
        assert!(retrieved.is_default);

        let list = repo.list_branches("repo-1").await?;
        assert_eq!(list.len(), 1);

        let mut updated = branch.clone();
        updated.head_commit = "def456".to_owned();
        repo.update_branch(&updated).await?;

        let after_update = repo.get_branch("branch-1").await?;
        assert_eq!(after_update.head_commit, "def456");

        repo.delete_branch("branch-1").await?;
        assert_not_found(&repo.get_branch("branch-1").await);
        return Ok(());
    }

    let wt = create_test_worktree("wt-1", "repo-1", "branch-1");
    repo.create_worktree(&wt).await?;

    let retrieved = repo.get_worktree("wt-1").await?;
    assert_eq!(retrieved.path, "/tmp/worktree-wt-1");

    let list = repo.list_worktrees("repo-1").await?;
    assert_eq!(list.len(), 1);

    let mut updated = wt.clone();
    updated.status = WorktreeStatus::InUse;
    updated.metadata.updated_at = 2_000_000;
    repo.update_worktree(&updated).await?;

    let after_update = repo.get_worktree("wt-1").await?;
    assert_eq!(after_update.status, WorktreeStatus::InUse);

    repo.delete_worktree("wt-1").await?;
    assert_not_found(&repo.get_worktree("wt-1").await);
    Ok(())
}

#[tokio::test]
async fn test_assignment_lifecycle() -> TestResult {
    let (repo, executor, _temp) = setup_repo().await?;
    let vcs_repo = create_test_repository("repo-1", "proj-1");
    repo.create_repository(&vcs_repo).await?;
    let branch = create_test_branch("branch-1", "repo-1", "main");
    repo.create_branch(&branch).await?;
    let wt = create_test_worktree("wt-1", "repo-1", "branch-1");
    repo.create_worktree(&wt).await?;
    seed_agent_session(executor.as_ref()).await?;

    let assignment = AgentWorktreeAssignment {
        id: "asgn-1".to_owned(),
        agent_session_id: "agent-1".to_owned(),
        worktree_id: "wt-1".to_owned(),
        assigned_at: 1_000_000,
        released_at: None,
    };
    repo.create_assignment(&assignment).await?;

    let retrieved = repo.get_assignment("asgn-1").await?;
    assert_eq!(retrieved.agent_session_id, "agent-1");
    assert!(retrieved.released_at.is_none());

    let list = repo.list_assignments_by_worktree("wt-1").await?;
    assert_eq!(list.len(), 1);

    repo.release_assignment("asgn-1", 2_000_000).await?;

    let after_release = repo.get_assignment("asgn-1").await?;
    assert_eq!(after_release.released_at, Some(2_000_000));
    Ok(())
}

#[tokio::test]
async fn test_org_isolation_repositories() -> TestResult {
    let (executor, _temp_dir) = setup_executor().await?;

    for org_id in &["org-A", "org-B"] {
        seed_isolated_org_scope(executor.as_ref(), org_id).await?;
    }

    let repo = SqliteVcsEntityRepository::new(executor);
    let vcs_repo = Repository {
        metadata: mcb_domain::entities::EntityMetadata {
            id: "repo-iso".to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        org_id: "org-A".to_owned(),
        project_id: "proj-org-A".to_owned(),
        name: "Org A Repo".to_owned(),
        url: "https://example.com/a.git".to_owned(),
        local_path: "/tmp/a".to_owned(),
        vcs_type: VcsType::Git,
    };
    repo.create_repository(&vcs_repo).await?;

    assert!(repo.get_repository("org-A", "repo-iso").await.is_ok());
    assert_not_found(&repo.get_repository("org-B", "repo-iso").await);
    assert!(
        repo.list_repositories("org-B", "proj-org-B")
            .await?
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn test_project_isolation_same_org_same_local_path() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;

    let mut repo_proj_1 = create_test_repository("repo-proj-1", "proj-1");
    repo_proj_1.local_path = "/tmp/shared-path".to_owned();
    let mut repo_proj_2 = create_test_repository("repo-proj-2", "proj-2");
    repo_proj_2.local_path = "/tmp/shared-path".to_owned();

    repo.create_repository(&repo_proj_1).await?;
    repo.create_repository(&repo_proj_2).await?;

    let list_proj_1 = repo.list_repositories(DEFAULT_ORG_ID, "proj-1").await?;
    let list_proj_2 = repo.list_repositories(DEFAULT_ORG_ID, "proj-2").await?;

    assert_eq!(list_proj_1.len(), 1);
    assert_eq!(list_proj_2.len(), 1);
    assert_eq!(list_proj_1[0].metadata.id, "repo-proj-1");
    assert_eq!(list_proj_2[0].metadata.id, "repo-proj-2");
    assert_eq!(list_proj_1[0].local_path, "/tmp/shared-path");
    assert_eq!(list_proj_2[0].local_path, "/tmp/shared-path");
    Ok(())
}

#[rstest]
#[case("branches")]
#[case("worktrees")]
#[tokio::test]
async fn list_entities_filter_by_repository(#[case] entity_kind: &str) -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;

    let repo1 = create_test_repository("repo-1", "proj-1");
    let repo2 = create_test_repository("repo-2", "proj-1");
    repo.create_repository(&repo1).await?;
    repo.create_repository(&repo2).await?;

    let b1 = create_test_branch("b1", "repo-1", "main");
    let b2 = create_test_branch("b2", "repo-2", "develop");
    repo.create_branch(&b1).await?;
    repo.create_branch(&b2).await?;

    let wt1 = create_test_worktree("wt-1", "repo-1", "b1");
    let wt2 = create_test_worktree("wt-2", "repo-2", "b2");
    repo.create_worktree(&wt1).await?;
    repo.create_worktree(&wt2).await?;

    if entity_kind == "branches" {
        let list_1 = repo.list_branches("repo-1").await?;
        assert_eq!(list_1.len(), 1);
        assert_eq!(list_1[0].name, "main");

        let list_2 = repo.list_branches("repo-2").await?;
        assert_eq!(list_2.len(), 1);
        assert_eq!(list_2[0].name, "develop");
        return Ok(());
    }

    let list_1 = repo.list_worktrees("repo-1").await?;
    assert_eq!(list_1.len(), 1);
    assert_eq!(list_1[0].metadata.id, "wt-1");

    let list_2 = repo.list_worktrees("repo-2").await?;
    assert_eq!(list_2.len(), 1);
    assert_eq!(list_2[0].metadata.id, "wt-2");
    Ok(())
}
