//! Shared test helpers for VCS tests.

use mcb_domain::ports::VcsProvider;
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};
use mcb_domain::test_utils::TestResult;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tokio::fs::write as tokio_write;

pub fn vcs_provider() -> TestResult<std::sync::Arc<dyn VcsProvider>> {
    Ok(resolve_vcs_provider(&VcsProviderConfig::new("git"))?)
}

pub fn run_git(dir: &Path, args: &[&str]) -> TestResult<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("git {args:?} failed").into())
    }
}

pub async fn create_test_repo() -> TestResult<TempDir> {
    let dir = TempDir::new()?;

    run_git(dir.path(), &["init"])?;
    run_git(dir.path(), &["config", "user.email", "test@example.com"])?;
    run_git(dir.path(), &["config", "user.name", "Test User"])?;

    tokio_write(dir.path().join("README.md"), "# Test Repo\n").await?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Initial commit"])?;

    Ok(dir)
}
