//! Unit tests for Git2 VCS provider.

use rstest::rstest;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};

use mcb_domain::ports::providers::VcsProvider;
use mcb_providers::git::Git2Provider;
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

#[rstest]
#[case(false)]
#[case(true)]
fn git2_provider_basics(#[case] check_object_safety: bool) {
    let provider = Git2Provider::new();
    assert!(
        !std::any::type_name::<Git2Provider>().is_empty(),
        "Git2Provider type exists"
    );
    if check_object_safety {
        fn _assert_object_safe(_: &dyn VcsProvider) {}
        _assert_object_safe(&provider);
        let _erased: &dyn VcsProvider = &provider;
        assert_eq!(
            std::mem::size_of::<&dyn VcsProvider>(),
            2 * std::mem::size_of::<usize>(),
            "trait object reference should be a fat pointer"
        );
    }
}

#[tokio::test]
async fn open_repository() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();

    let repo = provider.open_repository(dir.path()).await?;
    assert!(!repo.id().as_str().is_empty());
    assert!(!repo.branches().is_empty());
    Ok(())
}

#[rstest]
#[case("/nonexistent/path")]
#[case("/this/definitely/does/not/exist")]
#[tokio::test]
async fn open_repository_not_found(#[case] repo_path: &str) {
    let provider = Git2Provider::new();
    let result = provider.open_repository(Path::new(repo_path)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn repository_id_stable() -> TestResult<()> {
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

#[rstest]
#[case("branches")]
#[case("files")]
#[tokio::test]
async fn list_repository_entities(#[case] entity_kind: &str) -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    if entity_kind == "branches" {
        let branches = provider.list_branches(&repo).await?;
        assert!(!branches.is_empty());
        assert!(branches.iter().any(|b| b.is_default()));
    } else {
        let files = provider.list_files(&repo, repo.default_branch()).await?;
        assert!(files.iter().any(|f| f.to_string_lossy() == "README.md"));
    }
    Ok(())
}

#[rstest]
#[case(None, 1)]
#[case(Some(1), 1)]
#[tokio::test]
async fn commit_history_variants(
    #[case] limit: Option<usize>,
    #[case] expected_min_len: usize,
) -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let commits = provider
        .commit_history(&repo, repo.default_branch(), limit)
        .await?;

    assert!(commits.len() >= expected_min_len);
    if let Some(limit) = limit {
        assert_eq!(commits.len(), limit);
    }
    assert!(commits[0].message().contains("Initial commit"));
    Ok(())
}

#[rstest]
#[case("README.md", true)]
#[case("nonexistent.txt", false)]
#[tokio::test]
async fn read_file_variants(
    #[case] file_name: &str,
    #[case] should_succeed: bool,
) -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let content = provider
        .read_file(&repo, repo.default_branch(), Path::new(file_name))
        .await;

    if should_succeed {
        let content = content?;
        assert!(content.contains("# Test Repo"));
        return Ok(());
    }

    assert!(content.is_err());
    Ok(())
}

#[tokio::test]
async fn diff_refs() -> TestResult<()> {
    let dir = create_test_repo().await?;

    tokio_write(dir.path().join("new_file.txt"), "New content\n").await?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Second commit"])?;

    let provider = Git2Provider::new();
    let repo = provider.open_repository(dir.path()).await?;

    let commits = provider
        .commit_history(&repo, repo.default_branch(), Some(2))
        .await?;

    assert!(commits.len() >= 2);

    let diff = provider
        .diff_refs(&repo, commits[1].hash(), commits[0].hash())
        .await?;

    assert!(!diff.files.is_empty());
    assert!(
        diff.files
            .iter()
            .any(|f| f.path.to_string_lossy().contains("new_file"))
    );
    Ok(())
}
