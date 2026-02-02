use super::Git2Provider;
use mcb_domain::ports::providers::VcsProvider;
use std::path::Path;
use tempfile::TempDir;

fn create_test_repo() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to set git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to set git name");

    std::fs::write(dir.path().join("README.md"), "# Test Repo\n").expect("Failed to write file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .expect("Failed to add files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to commit");

    dir
}

#[tokio::test]
async fn test_open_repository() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();

    let repo = provider.open_repository(dir.path()).await;
    assert!(repo.is_ok());

    let repo = repo.unwrap();
    assert!(!repo.id.as_str().is_empty());
    assert!(!repo.branches.is_empty());
}

#[tokio::test]
async fn test_open_repository_not_found() {
    let provider = Git2Provider::new();
    let result = provider
        .open_repository(Path::new("/nonexistent/path"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_repository_id_stable() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();

    let repo1 = provider.open_repository(dir.path()).await.unwrap();
    let repo2 = provider.open_repository(dir.path()).await.unwrap();

    assert_eq!(
        provider.repository_id(&repo1),
        provider.repository_id(&repo2)
    );
}

#[tokio::test]
async fn test_list_branches() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let branches = provider.list_branches(&repo).await.unwrap();
    assert!(!branches.is_empty());
    assert!(branches.iter().any(|b| b.is_default));
}

#[tokio::test]
async fn test_commit_history() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let commits = provider
        .commit_history(&repo, &repo.default_branch, None)
        .await
        .unwrap();

    assert!(!commits.is_empty());
    assert!(commits[0].message.contains("Initial commit"));
}

#[tokio::test]
async fn test_commit_history_with_limit() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let commits = provider
        .commit_history(&repo, &repo.default_branch, Some(1))
        .await
        .unwrap();

    assert_eq!(commits.len(), 1);
}

#[tokio::test]
async fn test_list_files() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let files = provider
        .list_files(&repo, &repo.default_branch)
        .await
        .unwrap();
    assert!(files.iter().any(|f| f.to_string_lossy() == "README.md"));
}

#[tokio::test]
async fn test_read_file() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let content = provider
        .read_file(&repo, &repo.default_branch, Path::new("README.md"))
        .await
        .unwrap();

    assert!(content.contains("# Test Repo"));
}

#[tokio::test]
async fn test_read_file_not_found() {
    let dir = create_test_repo();
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await.unwrap();

    let result = provider
        .read_file(&repo, &repo.default_branch, Path::new("nonexistent.txt"))
        .await;

    assert!(result.is_err());
}
