/// Golden tests: Git-aware indexing with .mcp-context.toml configuration
/// Verifies multi-branch support, commit history, and configuration loading
use mcb_domain::utils::tests::fixtures::{
    create_test_mcp_server, golden_content_to_string, sample_codebase_path,
};
use mcb_server::args::{VcsAction, VcsArgs};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_index_repository_with_config() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    let result = server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::IndexRepository,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: None,
            target_branch: None,
            query: None,
            branches: Some(vec!["main".to_string()]),
            include_commits: Some(true),
            limit: Some(50),
            depth: Some(50),
            ignore_patterns: Some(vec!["*.log".to_string(), "target/".to_string()]),
        }))
        .await;

    assert!(result.is_ok(), "Repository indexing should succeed");
    let response = result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Response should not be error"
    );

    let text = golden_content_to_string(&response);
    assert!(
        text.contains("repository_id") || text.contains("indexed"),
        "Response should contain indexing results"
    );
}

#[tokio::test]
async fn golden_search_within_branch() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::IndexRepository,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: None,
            target_branch: None,
            query: None,
            branches: Some(vec!["main".to_string()]),
            include_commits: None,
            limit: None,
            depth: None,
            ignore_patterns: None,
        }))
        .await
        .expect("index");

    let search_result = server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::SearchBranch,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: Some("main".to_string()),
            target_branch: None,
            query: Some("function".to_string()),
            branches: None,
            include_commits: None,
            limit: Some(10),
            depth: None,
            ignore_patterns: None,
        }))
        .await;

    assert!(search_result.is_ok(), "Branch search should succeed");
}

#[tokio::test]
async fn golden_compare_branches() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::IndexRepository,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: None,
            target_branch: None,
            query: None,
            branches: Some(vec!["main".to_string()]),
            include_commits: None,
            limit: None,
            depth: None,
            ignore_patterns: None,
        }))
        .await
        .expect("index");

    let compare_result = server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::CompareBranches,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: Some("main".to_string()),
            target_branch: Some("main".to_string()),
            query: None,
            branches: None,
            include_commits: None,
            limit: None,
            depth: None,
            ignore_patterns: None,
        }))
        .await;

    assert!(compare_result.is_ok(), "Branch comparison should succeed");
}

#[tokio::test]
async fn golden_analyze_impact_with_depth() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::IndexRepository,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: None,
            target_branch: None,
            query: None,
            branches: Some(vec!["main".to_string()]),
            include_commits: Some(true),
            limit: None,
            depth: Some(100),
            ignore_patterns: None,
        }))
        .await
        .expect("index");

    let impact_result = server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::AnalyzeImpact,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: Some("main".to_string()),
            target_branch: Some("main".to_string()),
            query: None,
            branches: None,
            include_commits: None,
            limit: None,
            depth: Some(50),
            ignore_patterns: None,
        }))
        .await;

    assert!(impact_result.is_ok(), "Impact analysis should succeed");
}

#[tokio::test]
async fn golden_multi_branch_indexing() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    let branches = vec!["main".to_string(), "develop".to_string()];

    let result = server
        .vcs_handler()
        .handle(Parameters(VcsArgs {
            action: VcsAction::IndexRepository,
            path: Some(path.to_string_lossy().to_string()),
            repository_id: None,
            base_branch: None,
            target_branch: None,
            query: None,
            branches: Some(branches),
            include_commits: Some(true),
            limit: None,
            depth: Some(50),
            ignore_patterns: Some(vec!["*.log".to_string()]),
        }))
        .await;

    assert!(result.is_ok(), "Multi-branch indexing should succeed");
}
