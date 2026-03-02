use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::tests::utils::test_fixtures::{TEST_REPO_NAME, create_test_mcp_server};
use mcb_domain::utils::text::extract_text;
use mcb_server::args::SessionAction;
use mcb_server::args::SessionArgs;
use mcb_server::args::ValidateAction;
use mcb_server::args::ValidateArgs;
use mcb_server::args::VcsAction;
use mcb_server::args::VcsArgs;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use std::fs;
use std::process::Command;

#[rstest]
#[tokio::test]
async fn test_gap1_validate_list_rules_returns_populated_list() -> TestResult {
    let (server, _temp) = create_test_mcp_server().await?;
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

    let resp = result.expect("validate list-rules should succeed");
    assert!(!resp.is_error.unwrap_or(false));
    assert!(!resp.content.is_empty(), "Response content empty");

    let text = extract_text(&resp.content);
    let json_val: serde_json::Value = serde_json::from_str(&text).unwrap();

    // Expected format: { "count": N, "rules": [...] }
    let count = json_val
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    println!("Response JSON: {json_val}");
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
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_gap1_validate_list_rules_by_category_filter() -> TestResult {
    let (server, _temp) = create_test_mcp_server().await?;
    let validate_h = server.validate_handler();

    let result = validate_h
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::ListRules,
            path: None,
            category: Some("quality".to_owned()),
            scope: None,
            rules: None,
        }))
        .await;

    let resp = result.expect("validate list-rules with category filter should succeed");
    assert!(!resp.is_error.unwrap_or(false));
    assert!(!resp.content.is_empty(), "Response content empty");

    let text = extract_text(&resp.content);
    let json_val: serde_json::Value = serde_json::from_str(&text).unwrap();
    let rules = json_val
        .get("rules")
        .and_then(|v| v.as_array())
        .expect("Rules array should be present");

    assert!(!rules.is_empty(), "Filtered rules should not be empty");
    for rule in rules {
        assert_eq!(
            rule.get("category").and_then(|v| v.as_str()),
            Some("quality")
        );
    }
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_gap2_vcs_list_repositories_discovers_repos() -> TestResult {
    let (server, temp_dir) = create_test_mcp_server().await?;
    let vcs_h = server.vcs_handler();

    // Create a git repo in the temp dir
    let repo_path = temp_dir.path().join(TEST_REPO_NAME);
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
            action: VcsAction::ListRepositories,
            org_id: None,
            repo_id: None,
            repo_path: Some(temp_dir.path().to_string_lossy().into_owned()),
            base_branch: None,
            target_branch: None,
            query: None,
            branches: None,
            include_commits: None,
            depth: None,
            limit: None,
        }))
        .await;

    let resp = result.expect("vcs list-repositories should succeed");
    assert!(!resp.is_error.unwrap_or(false));
    assert!(!resp.content.is_empty(), "Response content empty");

    let text = extract_text(&resp.content);
    let json_val: serde_json::Value = serde_json::from_str(&text).unwrap();
    println!("VCS Response JSON: {json_val}");

    let count = json_val
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let repos = json_val.get("repositories").and_then(|v| v.as_array());

    assert!(count > 0, "Should find at least 1 repo");
    assert!(repos.is_some());

    // Check if our created repo is in the list
    let repo_strings: Vec<String> = repos
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(std::borrow::ToOwned::to_owned))
        .collect();

    let found = repo_strings.iter().any(|r| r.contains(TEST_REPO_NAME));
    assert!(
        found,
        "Created repo test-repo not found in list: {repo_strings:?}"
    );
    Ok(())
}

#[rstest]
#[case(None, true)]
#[case(Some(String::new()), true)]
#[case(Some("not_a_real_status".to_owned()), false)]
#[rstest]
#[tokio::test]
async fn test_gap3_session_list_status_handling(
    #[case] status: Option<String>,
    #[case] should_succeed: bool,
) -> TestResult {
    let (server, _temp) = create_test_mcp_server().await?;
    let session_h = server.session_handler();

    let result = session_h
        .handle(Parameters(SessionArgs {
            action: SessionAction::List,
            org_id: None,
            session_id: None,
            project_id: None,
            agent_type: None,
            data: None,
            worktree_id: None,
            parent_session_id: None,
            status,
            limit: Some(3),
        }))
        .await;

    if !should_succeed {
        assert!(
            result.is_err(),
            "Invalid status should return invalid_params"
        );
        return Ok(());
    }

    assert!(
        result.is_ok(),
        "Session list should accept optional/empty status"
    );
    let resp = result.unwrap();
    assert!(!resp.is_error.unwrap_or(false));

    let text = extract_text(&resp.content);
    let json_val: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(json_val.get("sessions").is_some());
    assert!(json_val.get("count").is_some());
    Ok(())
}
