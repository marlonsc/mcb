use crate::test_utils::test_fixtures::create_test_mcp_server;
use mcb_server::args::SessionAction;
use mcb_server::args::SessionArgs;
use mcb_server::args::ValidateAction;
use mcb_server::args::ValidateArgs;
use mcb_server::args::VcsAction;
use mcb_server::args::VcsArgs;
use rmcp::handler::server::wrapper::Parameters;
use std::fs;
use std::process::Command;

#[tokio::test]
async fn test_gap1_validate_list_rules_returns_populated_list() {
    let (server, _temp) = create_test_mcp_server().await;
    let validate_h = server.validate_handler();

    let result = validate_h
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::ListRules,
            path: None,
            category: None,
            scope: None,
            rules: None,
        }))
        .await;

    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(!resp.is_error.unwrap_or(false));
    assert!(!resp.content.is_empty(), "Response content empty");

    // Extract text from content
    let content_json = serde_json::to_value(&resp.content[0]).unwrap();
    let text = content_json
        .get("text")
        .expect("Content missing text field")
        .as_str()
        .expect("Text field not a string");

    let json_val: serde_json::Value = serde_json::from_str(text).unwrap();

    // Expected format: { "count": N, "rules": [...] }
    let count = json_val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    println!("Response JSON: {}", json_val);
    let rules = json_val.get("validators").and_then(|v| v.as_array());

    assert!(count > 0, "Validator count should be > 0 (GAP-1 Fix)");
    assert!(rules.is_some(), "Validators list should be present");
    assert!(
        !rules.unwrap().is_empty(),
        "Validators list should not be empty"
    );

    // Verify common validators are present
    let rule_names: Vec<&str> = rules.unwrap().iter().filter_map(|r| r.as_str()).collect();

    assert!(rule_names.contains(&"clean_architecture"));
    assert!(rule_names.contains(&"solid"));
}

#[tokio::test]
async fn test_gap2_vcs_list_repositories_discovers_repos() {
    let (server, temp_dir) = create_test_mcp_server().await;
    let vcs_h = server.vcs_handler();

    // Create a git repo in the temp dir
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();

    let _ = Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    fs::write(repo_path.join("README.md"), "# test-repo\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to git add");
    let _ = Command::new("git")
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .args(["commit", "-m", "init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create initial commit");

    let result = vcs_h
        .handle(Parameters(VcsArgs {
            org_id: None,
            action: VcsAction::ListRepositories,
            repo_id: None,
            repo_path: Some(temp_dir.path().to_string_lossy().to_string()),
            base_branch: None,
            target_branch: None,
            query: None,
            branches: None,
            include_commits: None,
            depth: None,
            org_id: None,
            limit: None,
        }))
        .await;

    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(!resp.is_error.unwrap_or(false));

    let content_json = serde_json::to_value(&resp.content[0]).unwrap();
    let text = content_json
        .get("text")
        .expect("Content missing text field")
        .as_str()
        .expect("Text field not a string");

    let json_val: serde_json::Value = serde_json::from_str(text).unwrap();
    println!("VCS Response JSON: {}", json_val);

    let count = json_val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    let repos = json_val.get("repositories").and_then(|v| v.as_array());

    assert!(count > 0, "Should find at least 1 repo");
    assert!(repos.is_some());

    // Check if our created repo is in the list
    let repo_strings: Vec<String> = repos
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let found = repo_strings.iter().any(|r| r.contains("test-repo"));
    assert!(
        found,
        "Created repo test-repo not found in list: {:?}",
        repo_strings
    );
}

#[tokio::test]
async fn test_gap3_session_list_works_without_agent_type() {
    let (server, _temp) = create_test_mcp_server().await;
    let session_h = server.session_handler();

    let result = session_h
        .handle(Parameters(SessionArgs {
            org_id: None,
            action: SessionAction::List,
            session_id: None,
            agent_type: None, // Omitted, should be allowed now
            data: None,
            project_id: None,
            org_id: None,
            worktree_id: None,
            status: None,
            limit: Some(3),
        }))
        .await;

    assert!(
        result.is_ok(),
        "Session list should succeed without agent_type"
    );
    let resp = result.unwrap();
    assert!(!resp.is_error.unwrap_or(false));

    let content_json = serde_json::to_value(&resp.content[0]).unwrap();
    let text = content_json
        .get("text")
        .expect("Content missing text field")
        .as_str()
        .expect("Text field not a string");

    let json_val: serde_json::Value = serde_json::from_str(text).unwrap();
    println!("Session Response JSON: {}", json_val);

    assert!(json_val.get("sessions").is_some());
    assert!(json_val.get("count").is_some());
}
