//! Shared test infrastructure for MCP tool command tests.
//!
//! Provides helpers for spawning the mcb binary and invoking MCP tools
//! via the rmcp client API through stdio transport.
//!
//! **CRITICAL**: the shared process lock prevents multiple mcb processes from
//! running simultaneously across test binaries and concurrent cargo test runs.

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::RunningService;
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// Per-operation timeout -- prevents any single MCP call or shutdown from hanging.
const OP_TIMEOUT: Duration =
    Duration::from_secs(mcb_utils::constants::testing::TEST_OP_TIMEOUT_SECS);

/// Server startup timeout -- longer to allow fastembed model download on first cold CI run.
/// The `AllMiniLML6V2` ONNX model (~90MB) must be downloaded from `HuggingFace` on first run.
/// Subsequent runs use the cached model and start in <3s.
const STARTUP_TIMEOUT: Duration =
    Duration::from_secs(mcb_utils::constants::testing::TEST_STARTUP_TIMEOUT_SECS);

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

pub struct McpTestClient {
    client: RunningService<RoleClient, ()>,
    _lock: crate::process_lock::ProcessLock,
}

impl Deref for McpTestClient {
    type Target = RunningService<RoleClient, ()>;

    fn deref(&self) -> &Self::Target {
        &self.client
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
    let unique_db = std::env::temp_dir().join(format!(
        "mcb-mcp-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let unique_db_str = unique_db.display().to_string().replace('\\', "/");
    register_temp_db(unique_db_str.clone());
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    cmd.current_dir(&workspace_root);
    cmd.arg("serve");
    cmd.arg("--stdio");
    cmd.env("LOCO_ENV", "test");
    cmd.env("DATABASE_URL", format!("sqlite://{unique_db_str}?mode=rwc"));
    for key in ["RUST_TEST_THREADS", "THREADS", "SCOPE", "RELEASE"] {
        cmd.env_remove(key);
    }
    cmd
}

/// Create an MCP client connected to a fresh mcb process via stdio.
///
/// Includes a timeout to prevent hanging if the server fails to start.
pub async fn create_client() -> Result<McpTestClient, Box<dyn std::error::Error>> {
    let lock = crate::process_lock::ProcessLock::acquire()?;
    let transport = TokioChildProcess::new(create_test_command())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let client = timeout(STARTUP_TIMEOUT, ().serve(transport))
        .await
        .map_err(|_| "Timeout: mcb server failed to start within 120s (fastembed model may be downloading on first run)")?
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(McpTestClient {
        client,
        _lock: lock,
    })
}

/// Gracefully shut down the MCP client with a timeout.
///
/// `cancel()` awaits the service `JoinHandle` which may block if the child process
/// doesn't exit on its own. Dropping the running service lets the rmcp transport
/// clean up the child before the process lock is released.
pub async fn shutdown_client(client: McpTestClient) {
    let McpTestClient { client, _lock } = client;
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
