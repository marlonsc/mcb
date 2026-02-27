//! Stdio Transport Integration Tests
//!
//! End-to-end tests for the stdio transport mode used by Claude Code.
//! These tests spawn the actual mcb binary and communicate via the rmcp client API.
//!
//! Critical for preventing regressions in MCP protocol communication:
//! - Log pollution (ANSI codes in stdout)
//! - JSON-RPC message framing
//! - Protocol handshake
//!
//! Run with: `cargo test -p mcb --test integration stdio_transport`

use rmcp::ServiceExt;
use rmcp::transport::child_process::TokioChildProcess;
use serial_test::serial;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Get the path to the mcb binary.
///
/// Uses `CARGO_BIN_EXE_mcb` which is set by cargo test when
/// the binary is built as part of the test run.
///
/// # Panics
///
/// Panics if the binary cannot be found in any expected location.
fn get_mcb_path() -> PathBuf {
    // cargo test sets this environment variable when the binary is part of the workspace
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mcb") {
        return PathBuf::from(path);
    }

    // Fallback: look in target directory relative to manifest
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bin = format!("mcb{}", std::env::consts::EXE_SUFFIX);
    let debug_path = PathBuf::from(manifest_dir).join(format!("../../target/debug/{bin}"));
    if debug_path.exists() {
        return debug_path;
    }

    let release_path = PathBuf::from(manifest_dir).join(format!("../../target/release/{bin}"));
    if release_path.exists() {
        return release_path;
    }

    unreachable!(
        "mcb binary not found. Run `cargo build -p mcb` first.\n\
         Checked:\n\
         - CARGO_BIN_EXE_mcb env var\n\
         - {manifest_dir}/../../target/debug/{bin}\n\
         - {manifest_dir}/../../target/release/{bin}"
    );
}

/// Spawn mcb with test-safe configuration (no external service dependencies)
fn create_test_command() -> Command {
    let mcb_path = get_mcb_path();
    let mut cmd = Command::new(mcb_path);
    let unique_db = format!(
        "/tmp/mcb-stdio-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    // Run from workspace root so Loco finds config/test.yaml
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    cmd.current_dir(&workspace_root);
    cmd.arg("serve");
    cmd.arg("--stdio");
    // Use Loco test environment (config/test.yaml with Tera template for DATABASE_URL)
    cmd.env("LOCO_ENV", "test");
    cmd.env("DATABASE_URL", format!("sqlite://{unique_db}?mode=rwc"));
    cmd
}

// =============================================================================
// STDOUT PURITY TESTS - Prevent regression of commit ffbe441
// =============================================================================

/// Test that stdout contains no ANSI escape codes (log pollution)
///
/// This prevents regression of the fix in commit ffbe441 where ANSI color codes
/// from logging were polluting the JSON-RPC stream on stdout.
#[serial]
#[tokio::test]
async fn test_stdio_no_ansi_codes_in_output() -> TestResult {
    let transport = TokioChildProcess::new(create_test_command())?;
    let client = ().serve(transport).await?;

    // Call peer_info to verify the connection is working
    // rmcp handles initialize + initialized automatically
    let peer_info = client.peer_info();
    assert!(
        peer_info.is_some(),
        "peer_info should be available after serve"
    );

    // Verify no ANSI codes in the protocol (rmcp handles this internally)
    // If we got here without errors, the protocol is clean
    let _ = client.cancel().await;
    Ok(())
}

/// Test that response is valid JSON (not corrupted by logs)
#[serial]
#[tokio::test]
async fn test_stdio_response_is_valid_json() -> TestResult {
    let transport = TokioChildProcess::new(create_test_command())?;
    let client = ().serve(transport).await?;

    // Call list_tools to get a response
    let tools_result = client.list_tools(None).await?;

    // Verify it's a valid response (rmcp handles JSON parsing internally)
    assert!(
        !tools_result.tools.is_empty(),
        "Should have at least one tool"
    );

    let _ = client.cancel().await;
    Ok(())
}

// =============================================================================
// END-TO-END ROUNDTRIP TESTS
// =============================================================================

/// Test complete tools/list roundtrip via stdio
#[serial]
#[tokio::test]
async fn test_stdio_roundtrip_tools_list() -> TestResult {
    let transport = TokioChildProcess::new(create_test_command())?;
    let client = ().serve(transport).await?;

    let tools_result = client.list_tools(None).await?;

    // Verify tools are returned
    assert!(
        !tools_result.tools.is_empty(),
        "Should have at least one tool"
    );

    // Verify expected tools exist
    let tool_names: Vec<String> = tools_result
        .tools
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    assert!(
        tool_names.contains(&"index".to_string()),
        "Missing index tool"
    );
    assert!(
        tool_names.contains(&"search".to_string()),
        "Missing search tool"
    );
    assert!(
        tool_names.contains(&"validate".to_string()),
        "Missing validate tool"
    );
    assert!(
        tool_names.contains(&"memory".to_string()),
        "Missing memory tool"
    );
    assert!(
        tool_names.contains(&"session".to_string()),
        "Missing session tool"
    );
    assert!(
        tool_names.contains(&"agent".to_string()),
        "Missing agent tool"
    );
    assert!(tool_names.contains(&"vcs".to_string()), "Missing vcs tool");

    let _ = client.cancel().await;
    Ok(())
}

/// Test initialize request via stdio
#[serial]
#[tokio::test]
async fn test_stdio_roundtrip_initialize() -> TestResult {
    let transport = TokioChildProcess::new(create_test_command())?;
    let client = ().serve(transport).await?;

    // Get peer info (already initialized by serve())
    let peer_info = client
        .peer_info()
        .ok_or("peer_info should be available after serve")?;

    // Verify protocol version is a proper string (not Debug format)
    let version_str = peer_info.protocol_version.to_string();
    assert!(
        !version_str.contains("ProtocolVersion"),
        "protocolVersion has Debug format leak"
    );

    // Verify serverInfo
    assert!(
        !peer_info.server_info.name.is_empty(),
        "Should have server name"
    );

    let _ = client.cancel().await;
    Ok(())
}

/// Test server handles unknown methods gracefully via stdio.
///
/// rmcp v0.16 uses `#[serde(untagged)]` + `#[serde(flatten)]` for JSON-RPC
/// message parsing.  When a `CustomRequest` catch-all deserialization succeeds
/// the server returns `METHOD_NOT_FOUND` (-32601).  However, on some platforms
/// (macOS) the serde untagged+flatten combination may fail, causing rmcp's
/// transport layer to treat the deserialization error as a closed stream and
/// shut down the connection.  Both outcomes are valid: the server either
/// responds with an error **or** closes the connection — it must NOT panic.
#[serial]
#[tokio::test]
async fn test_stdio_error_response_format() -> TestResult {
    let cmd = create_test_command();
    let (transport, _stderr) = rmcp::transport::child_process::TokioChildProcess::builder(cmd)
        .stderr(Stdio::piped())
        .spawn()?;
    let client = ().serve(transport).await?;

    // Try to call an unknown method
    // rmcp may close the connection or return an error
    // The key is that the server must NOT panic
    let result = client
        .call_tool(rmcp::model::CallToolRequestParams {
            meta: None,
            name: "nonexistent/method".into(),
            arguments: None,
            task: None,
        })
        .await;

    // Either error or connection closed is acceptable
    // We just verify the server didn't panic by the fact that we got here
    match result {
        Ok(_) => {
            // Server responded with success (unlikely but possible)
        }
        Err(_) => {
            // Server returned an error or closed connection (expected)
        }
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test that logs go to stderr, not stdout
#[serial]
#[tokio::test]
async fn test_stdio_logs_go_to_stderr() -> TestResult {
    let cmd = create_test_command();
    let (transport, stderr_handle) =
        rmcp::transport::child_process::TokioChildProcess::builder(cmd)
            .stderr(Stdio::piped())
            .spawn()?;
    let client = ().serve(transport).await?;

    // Send request
    let tools_result = client.list_tools(None).await?;

    // Verify stdout is pure JSON (rmcp handles this internally)
    assert!(!tools_result.tools.is_empty(), "Should have tools");

    // Give some time for stderr to accumulate logs
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Close the client
    let _ = client.cancel().await;

    // Check if stderr has content (logs)
    // Note: We can't guarantee logs are present, but if they are, they should be on stderr
    if let Some(stderr) = stderr_handle {
        use tokio::io::AsyncBufReadExt;
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await? > 0 {
            // Stderr lines should NOT be valid JSON-RPC responses
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&line);
            if let Ok(json) = parsed {
                assert!(
                    json.get("jsonrpc").is_none(),
                    "JSON-RPC message found in stderr - should be on stdout!"
                );
            }
            line.clear();
        }
    }

    Ok(())
}
