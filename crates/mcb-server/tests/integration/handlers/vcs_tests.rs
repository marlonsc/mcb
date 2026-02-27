use std::fs;
use std::process::Command;

use mcb_server::args::{VcsAction, VcsArgs};
use mcb_server::handlers::VcsHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::*;

use crate::utils::domain_services::create_real_domain_services;

async fn create_handler() -> Option<(VcsHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_real_domain_services().await?;
    Some((VcsHandler::new(state.mcp_server.vcs_provider()), temp_dir))
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

fn create_git_repo_fixture() -> Result<(tempfile::TempDir, String), std::io::Error> {
    let repo_dir = tempfile::tempdir()?;
    let repo_path = repo_dir.path().to_path_buf();

    fs::write(repo_path.join("README.md"), "# test\n")?;

    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo_path)
        .status()?;
    assert!(init.success(), "git init should succeed");

    let add = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .status()?;
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
        .status()?;
    assert!(commit.success(), "git commit should succeed");

    Ok((repo_dir, repo_path.to_string_lossy().to_string()))
}

#[rstest]
#[case(Some(10))]
#[case(Some(5))]
#[case(None)]
#[tokio::test]
async fn test_vcs_list_repositories_cases(
    #[case] limit: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return Ok(());
    };
    let (_repo_dir, repo_path) = create_git_repo_fixture()?;

    let mut args = base_vcs_args(VcsAction::ListRepositories);
    args.repo_path = Some(repo_path);
    args.limit = limit;

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("vcs handler should succeed for list repositories input");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_index_repository_success() -> Result<(), Box<dyn std::error::Error>> {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return Ok(());
    };
    let (_repo_dir, repo_path) = create_git_repo_fixture()?;

    let mut args = base_vcs_args(VcsAction::IndexRepository);
    args.repo_path = Some(repo_path);
    args.include_commits = Some(false);
    args.depth = Some(50);

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("vcs handler should succeed for repository indexing");
    assert!(!response.content.is_empty(), "response should have content");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_index_repository_with_repo_path() -> Result<(), rmcp::ErrorData> {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return Ok(());
    };

    let mut args = base_vcs_args(VcsAction::IndexRepository);
    args.repo_path = Some("/path/to/repo".to_owned());
    args.base_branch = Some("main".to_owned());

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("vcs handler should handle index request with repo_path");
    assert!(!response.content.is_empty(), "response should have content");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_analyze_impact_with_defaults() -> Result<(), rmcp::ErrorData> {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return Ok(());
    };

    let mut args = base_vcs_args(VcsAction::AnalyzeImpact);
    args.repo_id = Some("repo-123".to_owned());
    args.repo_path = Some("/path/to/repo".to_owned());
    args.target_branch = Some("feature/new-feature".to_owned());

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("vcs handler should handle analyze impact defaults");
    assert!(!response.content.is_empty(), "response should have content");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_analyze_impact_missing_repo_path() -> Result<(), rmcp::ErrorData> {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return Ok(());
    };

    let mut args = base_vcs_args(VcsAction::AnalyzeImpact);
    args.base_branch = Some("main".to_owned());
    args.target_branch = Some("feature/new-feature".to_owned());

    let result = handler.handle(Parameters(args)).await;

    assert!(
        result.is_err(),
        "Missing repo_path should return an Err for AnalyzeImpact"
    );
    let err = result.unwrap_err();
    assert!(
        err.message.contains("repo_path is required"),
        "error message should mention repo_path, got: {}",
        err.message
    );
    Ok(())
}
