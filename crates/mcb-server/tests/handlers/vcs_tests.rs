use std::sync::Arc;

use mcb_server::args::{VcsAction, VcsArgs};
use mcb_server::handlers::VcsHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::test_utils::mock_services::MockVcsProvider;

#[tokio::test]
async fn test_vcs_list_repositories_success() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::ListRepositories,
        org_id: None,
        repo_id: None,
        repo_path: None,
        base_branch: None,
        target_branch: None,
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: Some(10),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_vcs_list_repositories_with_limit() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::ListRepositories,
        org_id: None,
        repo_id: None,
        repo_path: None,
        base_branch: None,
        target_branch: None,
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: Some(5),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_vcs_list_repositories_no_limit() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::ListRepositories,
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
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_vcs_index_repository_success() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::IndexRepository,
        org_id: None,
        repo_id: Some("repo-123".to_string()),
        repo_path: Some("/path/to/repo".to_string()),
        base_branch: Some("main".to_string()),
        target_branch: None,
        query: None,
        branches: Some(vec!["main".to_string(), "develop".to_string()]),
        include_commits: Some(true),
        depth: Some(50),
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_vcs_index_repository_with_repo_path() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::IndexRepository,
        org_id: None,
        repo_id: None,
        repo_path: Some("/path/to/repo".to_string()),
        base_branch: Some("main".to_string()),
        target_branch: None,
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[tokio::test]
async fn test_vcs_analyze_impact_with_defaults() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::AnalyzeImpact,
        org_id: None,
        repo_id: Some("repo-123".to_string()),
        repo_path: Some("/path/to/repo".to_string()),
        base_branch: None,
        target_branch: Some("feature/new-feature".to_string()),
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[tokio::test]
async fn test_vcs_analyze_impact_missing_repo_path() {
    let mock_provider = MockVcsProvider::new();
    let handler = VcsHandler::new(Arc::new(mock_provider));

    let args = VcsArgs {
        action: VcsAction::AnalyzeImpact,
        org_id: None,
        repo_id: None,
        repo_path: None,
        base_branch: Some("main".to_string()),
        target_branch: Some("feature/new-feature".to_string()),
        query: None,
        branches: None,
        include_commits: None,
        depth: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing repo_path and repo_id should return error"
    );
}
