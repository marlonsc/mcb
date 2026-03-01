//! Unit tests for Git VCS provider — list/commit/read/diff/branch operations.

use rstest::rstest;
use std::path::Path;

use mcb_domain::test_utils::TestResult;
use tokio::fs::write as tokio_write;

use super::common::{create_test_repo, run_git, vcs_provider};

#[rstest]
#[case("branches")]
#[case("files")]
#[tokio::test]
async fn list_repository_entities(#[case] entity_kind: &str) -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = vcs_provider()?;
    let repo = provider.open_repository(dir.path()).await?;

    if entity_kind == "branches" {
        let branches = provider.list_branches(&repo).await?;
        assert!(!branches.is_empty());
        assert!(
            branches
                .iter()
                .any(mcb_domain::entities::VcsBranch::is_default)
        );
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
    let provider = vcs_provider()?;
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
    let provider = vcs_provider()?;
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

#[rstest]
#[tokio::test]
async fn diff_refs() -> TestResult<()> {
    let dir = create_test_repo().await?;

    tokio_write(dir.path().join("new_file.txt"), "New content\n").await?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Second commit"])?;

    let provider = vcs_provider()?;
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

#[rstest]
#[tokio::test]
async fn list_branches_skips_invalid_entries() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = vcs_provider()?;
    let repo = provider.open_repository(dir.path()).await?;

    // Create a valid branch
    run_git(dir.path(), &["checkout", "-b", "feature/test"])?;
    tokio_write(dir.path().join("feature.txt"), "Feature content\n").await?;
    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Feature commit"])?;

    // List branches - should include both main and feature/test
    let branches = provider.list_branches(&repo).await?;
    assert!(!branches.is_empty(), "Should have at least one branch");

    // Verify all branches have non-empty names and head commits
    for branch in &branches {
        assert!(!branch.name().is_empty(), "Branch name should not be empty");
        assert!(
            !branch.head_commit().is_empty(),
            "Head commit should not be empty"
        );
    }

    // Verify we have the feature branch
    assert!(
        branches.iter().any(|b| b.name() == "feature/test"),
        "Should have feature/test branch"
    );

    Ok(())
}
