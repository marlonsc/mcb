use std::fs;
use std::process::Command;

use mcb_server::args::{VcsAction, VcsArgs};
use mcb_server::handlers::VcsHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::*;

use crate::handlers::test_helpers::create_real_domain_services;

async fn create_handler() -> Option<(VcsHandler, tempfile::TempDir)> {
    let (services, temp_dir) = create_real_domain_services().await?;
    Some((VcsHandler::new(services.vcs_provider), temp_dir))
}

fn base_vcs_args(action: VcsAction) -> VcsArgs {
    VcsArgs {
        action,
        org_id: None,
        repo_id: None,
        repo_path: None,
        base_branch: None,
        target_branch: None,
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: None,
    }
}

fn create_git_repo_fixture() -> (tempfile::TempDir, String) {
    let repo_dir = tempfile::tempdir().expect("create temp repo dir");
    let repo_path = repo_dir.path().to_path_buf();

    fs::write(repo_path.join("README.md"), "# test\n").expect("write fixture file");

    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo_path)
        .status()
        .expect("run git init");
    assert!(init.success(), "git init should succeed");

    let add = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .status()
        .expect("run git add");
    assert!(add.success(), "git add should succeed");

    let commit = Command::new("git")
        .args([
            "-c",
            "user.name=Test User",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "init",
        ])
        .current_dir(&repo_path)
        .status()
        .expect("run git commit");
    assert!(commit.success(), "git commit should succeed");

    (repo_dir, repo_path.to_string_lossy().to_string())
}

#[rstest]
#[case(Some(10))]
#[case(Some(5))]
#[case(None)]
#[tokio::test]
async fn test_vcs_list_repositories_cases(#[case] limit: Option<u32>) {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };
    let (_repo_dir, repo_path) = create_git_repo_fixture();

    let mut args = base_vcs_args(VcsAction::ListRepositories);
    args.repo_path = Some(repo_path);
    args.limit = limit;

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[tokio::test]
async fn test_vcs_index_repository_success() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };
    let (_repo_dir, repo_path) = create_git_repo_fixture();

    let mut args = base_vcs_args(VcsAction::IndexRepository);
    args.repo_path = Some(repo_path);
    args.include_commits = Some(false);
    args.depth = Some(50);

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected successful response");
}

#[rstest]
#[tokio::test]
async fn test_vcs_index_repository_with_repo_path() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let mut args = base_vcs_args(VcsAction::IndexRepository);
    args.repo_path = Some("/path/to/repo".to_string());
    args.base_branch = Some("main".to_string());

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[rstest]
#[tokio::test]
async fn test_vcs_analyze_impact_with_defaults() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let mut args = base_vcs_args(VcsAction::AnalyzeImpact);
    args.repo_id = Some("repo-123".to_string());
    args.repo_path = Some("/path/to/repo".to_string());
    args.target_branch = Some("feature/new-feature".to_string());

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[rstest]
#[tokio::test]
async fn test_vcs_analyze_impact_missing_repo_path() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let mut args = base_vcs_args(VcsAction::AnalyzeImpact);
    args.base_branch = Some("main".to_string());
    args.target_branch = Some("feature/new-feature".to_string());

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing repo_path and repo_id should return error"
    );
}
