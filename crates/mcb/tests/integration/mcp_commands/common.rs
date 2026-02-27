//! Shared test infrastructure for MCP tool command tests.
//!
//! Provides helpers for spawning the mcb binary and invoking MCP tools
//! via the rmcp client API through stdio transport.
//!
//! **CRITICAL**: Every test using these helpers MUST be annotated with `#[serial]`
//! to prevent spawning multiple mcb processes simultaneously (each is ~50MB RSS).

use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::RunningService;
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// Per-operation timeout — prevents any single MCP call or shutdown from hanging.
const OP_TIMEOUT: Duration = Duration::from_secs(10);

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// --- Temp DB cleanup ---

static TEMP_DBS: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

fn get_temp_dbs() -> Arc<Mutex<Vec<String>>> {
    Arc::clone(TEMP_DBS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))))
}

fn register_temp_db(path: String) {
    let dbs = get_temp_dbs();
    if let Ok(mut dbs) = dbs.lock() {
        dbs.push(path);
    }
}

pub fn cleanup_temp_dbs() {
    let dbs = get_temp_dbs();
    if let Ok(mut dbs) = dbs.lock() {
        for db in dbs.drain(..) {
            let _ = std::fs::remove_file(&db);
        }
    }
}

// --- Binary discovery ---

fn get_mcb_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mcb") {
        return PathBuf::from(path);
    }
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
         Checked: CARGO_BIN_EXE_mcb, target/debug/{bin}, target/release/{bin}"
    );
}

// --- Test command & client ---

fn create_test_command() -> Command {
    let mcb_path = get_mcb_path();
    let mut cmd = Command::new(mcb_path);
    let unique_db = format!(
        "/tmp/mcb-mcp-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    register_temp_db(unique_db.clone());
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    cmd.current_dir(&workspace_root);
    cmd.arg("serve");
    cmd.arg("--stdio");
    cmd.env("LOCO_ENV", "test");
    cmd.env("DATABASE_URL", format!("sqlite://{unique_db}?mode=rwc"));
    cmd
}

/// Create an MCP client connected to a fresh mcb process via stdio.
///
/// Includes a timeout to prevent hanging if the server fails to start.
pub async fn create_client() -> Result<RunningService<RoleClient, ()>, Box<dyn std::error::Error>> {
    let transport = TokioChildProcess::new(create_test_command())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let client = timeout(OP_TIMEOUT, ().serve(transport))
        .await
        .map_err(|_| "Timeout: mcb server failed to start within 10s")?
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(client)
}

/// Gracefully shut down the MCP client with a timeout.
///
/// Wraps `client.cancel()` in a timeout so that if the child process doesn't
/// exit cleanly, the `Drop` impl on `ChildWithCleanup` will kill it.
pub async fn shutdown_client(client: RunningService<RoleClient, ()>) {
    // If cancel() hangs (child process stuck), the timeout fires and drops
    // the client, which triggers ChildWithCleanup::drop() → kill().
    let _ = timeout(Duration::from_secs(3), client.cancel()).await;
    cleanup_temp_dbs();
}

// --- Tool call helpers ---

fn json_args(value: Value) -> Option<serde_json::Map<String, Value>> {
    if let Value::Object(map) = value {
        Some(map)
    } else {
        unreachable!("json_args expects a JSON object")
    }
}

/// Call an MCP tool by name with JSON arguments.
///
/// Includes a timeout to prevent hanging on unresponsive tool calls.
pub async fn call_tool(
    client: &RunningService<RoleClient, ()>,
    tool_name: &str,
    arguments: Value,
) -> Result<CallToolResult, Box<dyn std::error::Error>> {
    let result = timeout(
        OP_TIMEOUT,
        client.call_tool(CallToolRequestParams {
            meta: None,
            name: tool_name.to_owned().into(),
            arguments: json_args(arguments),
            task: None,
        }),
    )
    .await
    .map_err(|_| format!("Timeout: tool '{tool_name}' did not respond within 10s"))?
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(result)
}

/// Extract all text content blocks from a tool result, joined by newlines.
pub fn extract_text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check if a tool result indicates an error.
pub fn is_error(result: &CallToolResult) -> bool {
    result.is_error.unwrap_or(false)
}

/// Assert that a tool call results in an error (either MCP-level or application-level).
///
/// The server may return errors as:
/// - `Err(McpError)` — JSON-RPC level (e.g., invalid params, unknown variant)
/// - `Ok(CallToolResult { is_error: true })` — application-level error
///
/// If `expected_keywords` is non-empty, at least one keyword must appear in the error.
pub fn assert_tool_error(
    result: Result<CallToolResult, Box<dyn std::error::Error>>,
    expected_keywords: &[&str],
) {
    match result {
        Err(e) => {
            if !expected_keywords.is_empty() {
                let msg = e.to_string().to_lowercase();
                assert!(
                    expected_keywords
                        .iter()
                        .any(|k| msg.contains(&k.to_lowercase())),
                    "Expected error containing one of {expected_keywords:?}, got: {e}"
                );
            }
        }
        Ok(r) if is_error(&r) => {
            if !expected_keywords.is_empty() {
                let text = extract_text(&r).to_lowercase();
                assert!(
                    expected_keywords
                        .iter()
                        .any(|k| text.contains(&k.to_lowercase())),
                    "Expected error containing one of {expected_keywords:?}, got: {}",
                    extract_text(&r)
                );
            }
        }
        Ok(r) => {
            panic!("Expected error, got success: {}", extract_text(&r));
        }
    }
}
