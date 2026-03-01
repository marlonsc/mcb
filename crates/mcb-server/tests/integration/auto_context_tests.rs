use std::path::Path;
use std::process::Command;

use mcb_infrastructure::resolution_context::create_default_hybrid_search_provider;
use mcb_providers::vcs::GitProvider;
use mcb_server::args::{SearchArgs, SearchResource};
use mcb_server::handlers::SearchHandler;
use mcb_server::tools::RuntimeDefaults;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use crate::utils::domain_services::create_real_domain_services;
use crate::utils::invariants::error_text;

const MCB_REPO_ROOT: &str = "/home/marlonsc/mcb";

fn probe_agent_program(
    env_overrides: &[(&str, &str)],
    env_removals: &[&str],
) -> Option<(std::process::ExitStatus, String, String)> {
    let exe = std::env::current_exe().ok()?;
    let mut command = Command::new(exe);
    command
        .arg("--exact")
        .arg("test_ide_probe_runtime_defaults")
        .arg("--nocapture")
        .env("MCB_IDE_PROBE", "1");

    for key in env_removals {
        command.env_remove(key);
    }
    for (key, value) in env_overrides {
        command.env(key, value);
    }

    let output = command.output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Some((output.status, stdout, stderr))
}

fn parse_agent_program(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("AGENT_PROGRAM=").map(ToOwned::to_owned))
}

#[rstest]
#[tokio::test]
async fn test_ide_probe_runtime_defaults() {
    if std::env::var("MCB_IDE_PROBE").ok().as_deref() != Some("1") {
        return;
    }

    let provider = GitProvider::new();
    let defaults =
        RuntimeDefaults::discover_from_path(&provider, Some(Path::new(MCB_REPO_ROOT)), None).await;

    let agent_program = defaults.agent_program.unwrap_or_default();
    println!("AGENT_PROGRAM={agent_program}");
}

#[rstest]
#[tokio::test]
async fn test_ide_detection_from_env_vars() {
    let ide_env_keys = [
        "CURSOR_TRACE_ID",
        "VSCODE_PID",
        "CLAUDE_CODE",
        "CLAUDE_SESSION_ID",
        "OPENCODE_SESSION_ID",
        "TERM_PROGRAM",
    ];

    let cursor_probe = probe_agent_program(&[("CURSOR_TRACE_ID", "test123")], &ide_env_keys);
    let Some((cursor_status, cursor_stdout, cursor_stderr)) = cursor_probe else {
        panic!("cursor probe process failed to start");
    };
    assert!(
        cursor_status.success(),
        "cursor probe failed: stdout={cursor_stdout} stderr={cursor_stderr}"
    );
    assert_eq!(
        parse_agent_program(&cursor_stdout).as_deref(),
        Some("cursor")
    );

    let vscode_probe = probe_agent_program(&[("VSCODE_PID", "12345")], &ide_env_keys);
    let Some((vscode_status, vscode_stdout, vscode_stderr)) = vscode_probe else {
        panic!("vscode probe process failed to start");
    };
    assert!(
        vscode_status.success(),
        "vscode probe failed: stdout={vscode_stdout} stderr={vscode_stderr}"
    );
    assert_eq!(
        parse_agent_program(&vscode_stdout).as_deref(),
        Some("vscode")
    );

    let claude_probe = probe_agent_program(&[("CLAUDE_CODE", "1")], &ide_env_keys);
    let Some((claude_status, claude_stdout, claude_stderr)) = claude_probe else {
        panic!("claude probe process failed to start");
    };
    assert!(
        claude_status.success(),
        "claude probe failed: stdout={claude_stdout} stderr={claude_stderr}"
    );
    assert_eq!(
        parse_agent_program(&claude_stdout).as_deref(),
        Some("claude-code")
    );

    let fallback_probe = probe_agent_program(&[], &ide_env_keys);
    let Some((fallback_status, fallback_stdout, fallback_stderr)) = fallback_probe else {
        panic!("fallback probe process failed to start");
    };
    assert!(
        fallback_status.success(),
        "fallback probe failed: stdout={fallback_stdout} stderr={fallback_stderr}"
    );
    assert_eq!(
        parse_agent_program(&fallback_stdout).as_deref(),
        Some("mcb-stdio")
    );
}

#[rstest]
#[tokio::test]
async fn test_search_without_collection_auto_resolves() {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        create_default_hybrid_search_provider(),
        state.mcp_server.indexing_service(),
    );

    let args = SearchArgs {
        query: "test query".to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: None,
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
        repo_id: Some("test-repo".to_owned()),
        repo_path: None,
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("search handler should return response");
    let message = error_text(&response);

    assert!(!message.contains("collection could not be resolved"));
}

#[rstest]
#[tokio::test]
async fn test_search_with_explicit_collection_still_works() {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        create_default_hybrid_search_provider(),
        state.mcp_server.indexing_service(),
    );

    let args = SearchArgs {
        query: "test query".to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some("invalid/collection".to_owned()),
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
        repo_id: Some("test-repo".to_owned()),
        repo_path: None,
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("search handler should return response");
    let message = error_text(&response);

    assert!(response.is_error.unwrap_or(false));
    assert!(message.contains("collection name contains invalid characters"));
}

#[rstest]
#[tokio::test]
async fn test_context_fields_populated_in_defaults() {
    let provider = GitProvider::new();
    let defaults =
        RuntimeDefaults::discover_from_path(&provider, Some(Path::new(MCB_REPO_ROOT)), None).await;

    assert!(defaults.session_id.is_some());
    assert!(defaults.repo_id.is_some());
    assert!(defaults.workspace_root.is_some());
    assert!(defaults.operator_id.is_some());
}

#[rstest]
#[tokio::test]
async fn test_org_id_from_git_remote() {
    let provider = GitProvider::new();
    let defaults =
        RuntimeDefaults::discover_from_path(&provider, Some(Path::new(MCB_REPO_ROOT)), None).await;

    assert!(defaults.org_id.is_some());
}
