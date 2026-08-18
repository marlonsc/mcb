//! Git test repository helpers.
//!
//! Centralized in `mcb-domain` so all crates that test VCS features
//! share the same git repo setup utilities.

use super::utils::TestResult;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// Execute a git command in the given directory.
///
/// # Errors
///
/// Returns an error if the git command fails.
pub fn run_git(dir: &Path, args: &[&str]) -> TestResult<()> {
    let mut command = Command::new("git");
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_CONFIG",
        "GIT_OBJECT_DIRECTORY",
        "GIT_COMMON_DIR",
    ] {
        command.env_remove(key);
    }
    let output = command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .arg("-c")
        .arg("core.hooksPath=/dev/null")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {args:?} failed: {stderr}").into())
    }
}

/// Create a temporary git repository with an initial commit.
///
/// Returns the `TempDir` — keep it alive for the test.
///
/// # Errors
///
/// Returns an error if git commands or file writes fail.
pub fn create_test_repo() -> TestResult<TempDir> {
    let dir = TempDir::new()?;

    run_git(dir.path(), &["init"])?;
    run_git(dir.path(), &["config", "user.email", "test@example.com"])?;
    run_git(dir.path(), &["config", "user.name", "Test User"])?;

    std::fs::write(dir.path().join("README.md"), "# Test Repo\n")?;

    run_git(dir.path(), &["add", "."])?;
    run_git(dir.path(), &["commit", "-m", "Initial commit"])?;

    Ok(dir)
}
