use super::Git2Provider;
use mcb_domain::ports::providers::VcsProvider;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tokio::fs::write as tokio_write;

type TestResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn run_git(dir: &Path, args: &[&str]) -> TestResult<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("git {:?} failed", args).into())
    }
}

async fn create_test_repo() -> TestResult<TempDir> {
    let dir = TempDir::new()?;

    run_git(dir.path(), &["init"])?;
    run_git(dir.path(), &["config", "user.email", "test@example.com"])?;
    run_git(dir.path(), &["config", "user.name", "Test User"])?;

    tokio_write(dir.path().join("README.md"), "# Test Repo\n").await?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Initial commit"])?;

    Ok(dir)
}

#[tokio::test]
async fn test_open_repository() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();

    let repo = provider.open_repository(dir.path()).await?;
    assert!(!repo.id.as_str().is_empty());
    assert!(!repo.branches.is_empty());
    Ok(())
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
async fn test_repository_id_stable() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();

    let repo1 = provider.open_repository(dir.path()).await?;
    let repo2 = provider.open_repository(dir.path()).await?;

    assert_eq!(
        provider.repository_id(&repo1),
        provider.repository_id(&repo2)
    );
    Ok(())
}

#[tokio::test]
async fn test_list_branches() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let branches = provider.list_branches(&repo).await?;
    assert!(!branches.is_empty());
    assert!(branches.iter().any(|b| b.is_default));
    Ok(())
}

#[tokio::test]
async fn test_commit_history() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let commits = provider
        .commit_history(&repo, &repo.default_branch, None)
        .await?;

    assert!(!commits.is_empty());
    assert!(commits[0].message.contains("Initial commit"));
    Ok(())
}

#[tokio::test]
async fn test_commit_history_with_limit() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let commits = provider
        .commit_history(&repo, &repo.default_branch, Some(1))
        .await?;

    assert_eq!(commits.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_list_files() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let files = provider.list_files(&repo, &repo.default_branch).await?;
    assert!(files.iter().any(|f| f.to_string_lossy() == "README.md"));
    Ok(())
}

#[tokio::test]
async fn test_read_file() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let content = provider
        .read_file(&repo, &repo.default_branch, Path::new("README.md"))
        .await?;

    assert!(content.contains("# Test Repo"));
    Ok(())
}

#[tokio::test]
async fn test_read_file_not_found() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let result = provider
        .read_file(&repo, &repo.default_branch, Path::new("nonexistent.txt"))
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_diff_refs() -> TestResult<()> {
    let dir = create_test_repo().await?;

    tokio_write(dir.path().join("new_file.txt"), "New content\n").await?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Second commit"])?;

    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let commits = provider
        .commit_history(&repo, &repo.default_branch, Some(2))
        .await?;

    assert!(commits.len() >= 2);

    let diff = provider
        .diff_refs(&repo, &commits[1].hash, &commits[0].hash)
        .await?;

    assert!(!diff.files.is_empty());
    assert!(
        diff.files
            .iter()
            .any(|f| f.path.to_string_lossy().contains("new_file"))
    );
    Ok(())
}
