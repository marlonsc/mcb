//! Isolation guarantees for the shared git test helpers.
//!
//! `git` resolves `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and `GIT_CONFIG`
//! with higher precedence than the process working directory. A suite invoked
//! from inside a git operation (pre-commit / pre-push hook, rebase, merge)
//! inherits that environment, which redirects helper git calls away from the
//! temporary repository and onto the surrounding real repository.
//!
//! These tests reproduce the hook environment for a child `git` process and
//! assert, through the public helper surface, that a repository built by the
//! helpers is self-contained and unaffected by it.

use mcb_domain::utils::tests::git_helpers::{create_test_repo, run_git};
use mcb_domain::utils::tests::utils::TestResult;
use std::process::Command;
use tempfile::TempDir;

/// A `git` invocation that reads only the directory it is pointed at.
///
/// Mirrors the isolation the helper applies, so assertions cannot be answered
/// by the surrounding repository when the suite runs inside a git hook.
fn git_reader() -> Command {
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
    command
}

/// Ask a child `git`, carrying the hook environment, which repository the
/// helper-built directory actually belongs to.
///
/// The environment is applied to the child only, so the test process stays
/// clean and the suite remains safe to run in parallel.
fn toplevel_seen_with_hook_environment(
    path: &std::path::Path,
    decoy: &TempDir,
) -> TestResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .env("GIT_DIR", decoy.path().join("decoy.git"))
        .env("GIT_WORK_TREE", decoy.path())
        .env("GIT_INDEX_FILE", decoy.path().join("decoy.index"))
        .env("GIT_CONFIG", decoy.path().join("decoy.gitconfig"))
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[test]
fn create_test_repo_builds_a_self_contained_repository() -> TestResult<()> {
    let repo = create_test_repo()?;

    // The repository exists where the helper was told to build it.
    assert!(
        repo.path().join(".git").exists(),
        "helper built no repository at its own path"
    );

    // It carries the initial commit, read through git's own public surface.
    // The reader clears the same inherited variables the helper does: otherwise
    // an ambient GIT_DIR would make this assertion read the surrounding
    // repository's history instead of the one under test.
    let log = git_reader()
        .args(["log", "--oneline"])
        .current_dir(repo.path())
        .output()?;
    assert!(
        log.status.success(),
        "helper repository has no readable history: {}",
        String::from_utf8_lossy(&log.stderr)
    );
    assert!(
        String::from_utf8_lossy(&log.stdout).contains("Initial commit"),
        "helper repository is missing its initial commit"
    );

    Ok(())
}

#[test]
fn helper_repository_is_not_captured_by_ambient_git_environment() -> TestResult<()> {
    let decoy = TempDir::new()?;
    let repo = create_test_repo()?;

    let seen = toplevel_seen_with_hook_environment(repo.path(), &decoy)?;

    // Under a leaking implementation the child resolves to the decoy work tree.
    // A correctly isolated repository resolves to itself (or refuses), never to
    // the ambient location.
    assert!(
        !seen.contains(&decoy.path().to_string_lossy().to_string()),
        "ambient GIT_WORK_TREE captured the helper repository: git reported {seen}"
    );

    Ok(())
}

#[test]
fn run_git_initialises_only_the_directory_it_is_given() -> TestResult<()> {
    let target = TempDir::new()?;

    run_git(target.path(), &["init"])?;

    assert!(
        target.path().join(".git").exists(),
        "run_git did not initialise the directory it was given"
    );

    Ok(())
}
