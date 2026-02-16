//! Stdio Transport Integration Tests
//!
//! End-to-end tests for the stdio transport mode used by Claude Code.
//! These tests spawn the actual mcb binary and communicate via stdin/stdout.
//!
//! Critical for preventing regressions in MCP protocol communication:
//! - Log pollution (ANSI codes in stdout)
//! - JSON-RPC message framing
//! - Protocol handshake
//!
//! Run with: `cargo test -p mcb-server --test integration stdio_transport`

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// RAII guard that ensures a child process is killed and waited on when dropped.
/// Prevents zombie processes on early `?` returns.
struct ChildGuard(Option<std::process::Child>);

impl ChildGuard {
    fn new(child: std::process::Child) -> Self {
        Self(Some(child))
    }

    /// Take the inner child out of the guard (for explicit cleanup).
    fn inner_mut(&mut self) -> &mut std::process::Child {
        self.0
            .as_mut()
            .unwrap_or_else(|| unreachable!("child already taken"))
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

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
    let debug_path = PathBuf::from(manifest_dir).join("../../target/debug/mcb");
    if debug_path.exists() {
        return debug_path;
    }

    let release_path = PathBuf::from(manifest_dir).join("../../target/release/mcb");
    if release_path.exists() {
        return release_path;
    }

    unreachable!(
        "mcb binary not found. Run `cargo build -p mcb-server` first.\n\
         Checked:\n\
         - CARGO_BIN_EXE_mcb env var\n\
         - {manifest_dir}/../../target/debug/mcb\n\
         - {manifest_dir}/../../target/release/mcb"
    );
}

/// Spawn mcb with test-safe configuration (no external service dependencies)
fn create_test_command(mcb_path: &PathBuf) -> Command {
    let mut cmd = Command::new(mcb_path);
    let config_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/smoke-test.toml");
    let unique_db = format!(
        "/tmp/mcb-stdio-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    cmd.arg("serve");
    cmd.arg("--config").arg(config_path);
    cmd.env(
        "MCP__PROVIDERS__DATABASE__CONFIGS__DEFAULT__PATH",
        unique_db,
    );
    cmd
}

/// Helper to spawn mcb binary with stdio transport.
///
/// # Panics
///
/// Panics if the process cannot be spawned.
fn spawn_mcb_stdio() -> ChildGuard {
    let mcb_path = get_mcb_path();

    let child = create_test_command(&mcb_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| unreachable!("Failed to spawn mcb at {mcb_path:?}: {e}"));

    ChildGuard::new(child)
}

/// Send a JSON-RPC request and read the response.
///
/// # Errors
///
/// Returns an error if writing, flushing, reading, or parsing fails.
fn send_request_get_response(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    request: &serde_json::Value,
) -> TestResult<serde_json::Value> {
    // Send request with newline delimiter
    let request_str = serde_json::to_string(request)?;
    writeln!(stdin, "{request_str}")?;
    stdin.flush()?;

    // Read response line
    let mut response_line = String::new();
    let n = stdout.read_line(&mut response_line)?;
    assert!(
        n > 0,
        "EOF reading stdout - server likely crashed. Check stderr."
    );

    let val: serde_json::Value = serde_json::from_str(&response_line)?;
    Ok(val)
}

/// Create the MCP initialize request required to start a session
fn create_initialize_request(id: i64) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": id
    })
}

/// Send the initialized notification (required after initialize response).
///
/// # Errors
///
/// Returns an error if writing or flushing fails.
fn send_initialized_notification(stdin: &mut std::process::ChildStdin) -> TestResult {
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notification_str = serde_json::to_string(&notification)?;
    writeln!(stdin, "{notification_str}")?;
    stdin.flush()?;
    Ok(())
}

/// Initialize the MCP session (required before any other requests).
///
/// # Errors
///
/// Returns an error if the initialize handshake fails.
fn initialize_mcp_session(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
) -> TestResult<serde_json::Value> {
    let init_request = create_initialize_request(0);
    let response = send_request_get_response(stdin, stdout, &init_request)?;

    // Send initialized notification
    send_initialized_notification(stdin)?;

    Ok(response)
}

// =============================================================================
// STDOUT PURITY TESTS - Prevent regression of commit ffbe441
// =============================================================================

/// Test that stdout contains no ANSI escape codes (log pollution)
///
/// This prevents regression of the fix in commit ffbe441 where ANSI color codes
/// from logging were polluting the JSON-RPC stream on stdout.
#[test]
fn test_stdio_no_ansi_codes_in_output() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdout_reader = BufReader::new(stdout);

    // Send initialize request (required by MCP protocol)
    let request = create_initialize_request(1);

    let request_str = serde_json::to_string(&request)?;
    writeln!(stdin, "{request_str}")?;
    stdin.flush()?;

    // Read response
    let mut response_line = String::new();
    stdout_reader.read_line(&mut response_line)?;

    // CRITICAL: Check for ANSI escape codes
    // \x1b[ is the start of ANSI escape sequences
    assert!(
        !response_line.contains("\x1b["),
        "ANSI escape codes found in stdout! This breaks JSON-RPC protocol.\nResponse: {response_line:?}"
    );

    // Also check for common ANSI codes
    assert!(
        !response_line.contains("\x1b"),
        "Escape character found in stdout! Response: {response_line:?}"
    );

    Ok(())
    // ChildGuard drop handles kill + wait
}

/// Test that response is valid JSON (not corrupted by logs)
#[test]
fn test_stdio_response_is_valid_json() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader)?;

    // Send tools/list request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    writeln!(stdin, "{request_str}")?;
    stdin.flush()?;

    // Read response
    let mut response_line = String::new();
    stdout_reader.read_line(&mut response_line)?;

    // Verify it's valid JSON
    let response: serde_json::Value = serde_json::from_str(&response_line)?;

    // Verify it has JSON-RPC structure
    assert_eq!(
        response.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "Response missing jsonrpc field"
    );

    Ok(())
}

// =============================================================================
// END-TO-END ROUNDTRIP TESTS
// =============================================================================

/// Test complete tools/list roundtrip via stdio
#[test]
fn test_stdio_roundtrip_tools_list() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader)?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 42
    });

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request)?;

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 42);
    assert!(
        response["error"].is_null(),
        "Unexpected error: {:?}",
        response["error"]
    );

    // Verify tools are returned
    let result = &response["result"];
    assert!(result["tools"].is_array(), "tools should be an array");

    let tools = result["tools"]
        .as_array()
        .ok_or("tools should be an array")?;
    assert!(!tools.is_empty(), "Should have at least one tool");

    // Verify expected tools exist
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(tool_names.contains(&"index"), "Missing index tool");
    assert!(tool_names.contains(&"search"), "Missing search tool");
    assert!(tool_names.contains(&"validate"), "Missing validate tool");
    assert!(tool_names.contains(&"memory"), "Missing memory tool");
    assert!(tool_names.contains(&"session"), "Missing session tool");
    assert!(tool_names.contains(&"agent"), "Missing agent tool");
    assert!(tool_names.contains(&"vcs"), "Missing vcs tool");

    Ok(())
}

/// Test initialize request via stdio
#[test]
fn test_stdio_roundtrip_initialize() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdout_reader = BufReader::new(stdout);

    let request = create_initialize_request(1);

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request)?;

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response["error"].is_null(),
        "Unexpected error: {:?}",
        response["error"]
    );

    let result = &response["result"];

    // Verify protocol version is a proper string (not Debug format)
    let version = &result["protocolVersion"];
    assert!(version.is_string(), "protocolVersion should be a string");
    let version_str = version.as_str().ok_or("protocolVersion not a string")?;
    assert!(
        !version_str.contains("ProtocolVersion"),
        "protocolVersion has Debug format leak"
    );

    // Verify serverInfo
    assert!(result["serverInfo"].is_object(), "Should have serverInfo");
    assert!(
        result["serverInfo"]["name"].is_string(),
        "Should have server name"
    );

    Ok(())
}

/// Test error response via stdio (unknown method)
#[test]
fn test_stdio_error_response_format() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader)?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "nonexistent/method",
        "id": 99
    });

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request)?;

    // Verify error response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 99);
    assert!(response["result"].is_null(), "Should not have result");
    assert!(response["error"].is_object(), "Should have error object");

    let error = &response["error"];
    assert!(error["code"].is_i64(), "Error should have numeric code");
    assert!(error["message"].is_string(), "Error should have message");

    Ok(())
}

// =============================================================================
// LOGGING TO STDERR TEST
// =============================================================================

/// Test that logs go to stderr, not stdout
#[test]
fn test_stdio_logs_go_to_stderr() -> TestResult {
    let mut guard = spawn_mcb_stdio();
    let child = guard.inner_mut();

    let mut stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    let mut stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader)?;

    // Send request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    writeln!(stdin, "{request_str}")?;
    stdin.flush()?;

    // Read stdout response
    let mut response_line = String::new();
    stdout_reader.read_line(&mut response_line)?;

    // Stdout should be pure JSON
    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    assert_eq!(response["jsonrpc"], "2.0");

    // Give some time for stderr to accumulate logs
    std::thread::sleep(Duration::from_millis(100));

    // Terminate process before reading stderr to avoid blocking on open pipe
    drop(stdin);
    // ChildGuard handles kill + wait on drop, but we need the stderr reader to work
    // so kill explicitly here before reading
    let _ = child.kill();
    let _ = child.wait();

    // Check if stderr has content (logs)
    // Note: We can't guarantee logs are present, but if they are, they should be on stderr
    let stderr_lines: Vec<_> = stderr_reader.lines().take(10).collect();

    // If there are any stderr lines with log-like content, that's expected behavior
    // The key assertion is that stdout ONLY has JSON
    for line in stderr_lines.into_iter().flatten() {
        // Stderr lines should NOT be valid JSON-RPC responses
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&line);
        if let Ok(json) = parsed {
            assert!(
                json.get("jsonrpc").is_none(),
                "JSON-RPC message found in stderr - should be on stdout!"
            );
        }
    }

    Ok(())
}
