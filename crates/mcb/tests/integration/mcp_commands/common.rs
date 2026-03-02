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

pub use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
pub use mcb_domain::utils::tests::utils::TestResult;

/// Per-operation timeout -- prevents any single MCP call or shutdown from hanging.
const OP_TIMEOUT: Duration = Duration::from_secs(10);

/// Server startup timeout -- longer to allow fastembed model download on first cold CI run.
/// The `AllMiniLML6V2` ONNX model (~90MB) must be downloaded from `HuggingFace` on first run.
/// Subsequent runs use the cached model and start in <3s.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(120);

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

// --- Child PID tracking (ensures processes are killed after cancel() times out) ---

static CHILD_PIDS: OnceLock<Arc<Mutex<Vec<u32>>>> = OnceLock::new();

fn get_child_pids() -> Arc<Mutex<Vec<u32>>> {
    Arc::clone(CHILD_PIDS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))))
}

fn register_child_pid(pid: u32) {
    if let Ok(mut pids) = get_child_pids().lock() {
        pids.push(pid);
    }
}

fn kill_registered_children() {
    if let Ok(mut pids) = get_child_pids().lock() {
        for pid in pids.drain(..) {
            let _ = std::process::Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status();
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
    // Track the PID so we can forcefully kill the process if cancel() hangs.
    if let Some(pid) = transport.id() {
        register_child_pid(pid);
    }
    let client = timeout(STARTUP_TIMEOUT, ().serve(transport))
        .await
        .map_err(|_| "Timeout: mcb server failed to start within 120s (fastembed model may be downloading on first run)")?
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(client)
}

/// Gracefully shut down the MCP client with a timeout, then kill any stray child.
///
/// `cancel()` awaits the service `JoinHandle` which may block if the child process
/// doesn't exit on its own. After the 3-second timeout we forcefully kill any
/// registered child PIDs so the next serial test can start a fresh process.
pub async fn shutdown_client(client: RunningService<RoleClient, ()>) {
    let _ = timeout(Duration::from_secs(3), client.cancel()).await;
    // Forcefully kill any remaining child processes.
    kill_registered_children();
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
