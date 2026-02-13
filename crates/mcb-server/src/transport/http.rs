//! HTTP Transport for MCP
//!
//! Implements MCP protocol over HTTP using Server-Sent Events (SSE).
//! This transport allows web clients to connect to the MCP server.
//!
//! # Architecture
//!
//! This transport consolidates all HTTP endpoints into a single port:
//! - MCP protocol endpoints (`/mcp`, `/events`)
//! - Health/readiness probes (`/healthz`, `/readyz`)
//! - Admin API endpoints (`/health`, `/config`, `/collections`, etc.)
//! - Prometheus metrics (`/metrics`)
//!
//! # Supported MCP Methods
//!
//! | Method | Description |
//! | -------- | ------------- |
//! | `initialize` | Initialize the MCP session |
//! | `tools/list` | List available tools |
//! | `tools/call` | Call a tool with arguments |
//! | `ping` | Health check |
//!
//! # Example
//!
//! ```text
//! POST /mcp HTTP/1.1
//! Content-Type: application/json
//!
//! {
//!     "jsonrpc": "2.0",
//!     "method": "tools/list",
//!     "id": 1
//! }
//! ```
//!
//! # Migration Note
//! Consolidated Admin API into single port in v0.2.0.

use std::net::SocketAddr;
use std::sync::Arc;

use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Build, Request, Response, Rocket, State, get, post, routes};
use tracing::{error, info};

use super::types::{McpRequest, McpResponse};
use crate::McpServer;
use crate::admin::auth::AdminAuthConfig;
use crate::admin::browse_handlers::BrowseState;
use crate::admin::handlers::AdminState;
use crate::admin::routes::admin_rocket;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{ToolExecutionContext, ToolHandlers, create_tool_list, route_tool_call};
use mcb_infrastructure::config::ConfigLoader;

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Host address to bind the HTTP server (e.g., "127.0.0.1", "0.0.0.0")
    pub host: String,
    /// Port number for the HTTP server
    pub port: u16,
    /// Whether to enable CORS headers for cross-origin requests
    pub enable_cors: bool,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("HttpTransportConfig::default requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port: config.server.network.port,
            enable_cors: config.server.cors.cors_enabled,
        }
    }
}

impl HttpTransportConfig {
    /// Create config for localhost with specified port
    pub fn localhost(port: u16) -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("HttpTransportConfig::localhost requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port,
            enable_cors: config.server.cors.cors_enabled,
        }
    }

    /// Get the socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid host/port in configuration")
    }
}

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    /// Shared reference to the MCP server instance
    pub server: Arc<McpServer>,
}

#[derive(Debug, Clone)]
struct BridgeProvenance {
    workspace_root: Option<String>,
    repo_path: Option<String>,
    repo_id: Option<String>,
    session_id: Option<String>,
    parent_session_id: Option<String>,
    project_id: Option<String>,
    worktree_id: Option<String>,
    operator_id: Option<String>,
    machine_id: Option<String>,
    agent_program: Option<String>,
    model_id: Option<String>,
    delegated: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BridgeProvenance {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let workspace_root = request
            .headers()
            .get_one("X-Workspace-Root")
            .map(ToOwned::to_owned);
        let repo_path = request
            .headers()
            .get_one("X-Repo-Path")
            .map(ToOwned::to_owned);
        let repo_id = request
            .headers()
            .get_one("X-Repo-Id")
            .map(ToOwned::to_owned);
        let session_id = request
            .headers()
            .get_one("X-Session-Id")
            .map(ToOwned::to_owned);
        let parent_session_id = request
            .headers()
            .get_one("X-Parent-Session-Id")
            .map(ToOwned::to_owned);
        let project_id = request
            .headers()
            .get_one("X-Project-Id")
            .map(ToOwned::to_owned);
        let worktree_id = request
            .headers()
            .get_one("X-Worktree-Id")
            .map(ToOwned::to_owned);
        let operator_id = request
            .headers()
            .get_one("X-Operator-Id")
            .map(ToOwned::to_owned);
        let machine_id = request
            .headers()
            .get_one("X-Machine-Id")
            .map(ToOwned::to_owned);
        let agent_program = request
            .headers()
            .get_one("X-Agent-Program")
            .map(ToOwned::to_owned);
        let model_id = request
            .headers()
            .get_one("X-Model-Id")
            .map(ToOwned::to_owned);
        let delegated = request
            .headers()
            .get_one("X-Delegated")
            .map(ToOwned::to_owned);

        Outcome::Success(Self {
            workspace_root,
            repo_path,
            repo_id,
            session_id,
            parent_session_id,
            project_id,
            worktree_id,
            operator_id,
            machine_id,
            agent_program,
            model_id,
            delegated,
        })
    }
}

/// HTTP transport server with optional admin API integration
pub struct HttpTransport {
    config: HttpTransportConfig,
    state: HttpTransportState,
    admin_state: Option<AdminState>,
    auth_config: Option<Arc<AdminAuthConfig>>,
    browse_state: Option<BrowseState>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(config: HttpTransportConfig, server: Arc<McpServer>) -> Self {
        Self {
            config,
            state: HttpTransportState { server },
            admin_state: None,
            auth_config: None,
            browse_state: None,
        }
    }

    /// Add admin API state for consolidated single-port operation
    pub fn with_admin(
        mut self,
        admin_state: AdminState,
        auth_config: Arc<AdminAuthConfig>,
        browse_state: Option<BrowseState>,
    ) -> Self {
        self.admin_state = Some(admin_state);
        self.auth_config = Some(auth_config);
        self.browse_state = browse_state;
        self
    }

    /// Build the Rocket application with MCP and optional Admin routes.
    ///
    /// Delegates all admin/web routes to [`admin_rocket()`] as the single source
    /// of truth, then layers MCP-specific routes on top.
    pub fn rocket(&self) -> Rocket<Build> {
        let mut rocket = if let Some(ref admin_state) = self.admin_state {
            let auth_config = self
                .auth_config
                .clone()
                .unwrap_or_else(|| Arc::new(AdminAuthConfig::default()));
            admin_rocket(admin_state.clone(), auth_config, self.browse_state.clone())
        } else {
            rocket::custom(rocket::Config::figment())
        };

        rocket = rocket
            .manage(self.state.clone())
            .mount("/", routes![handle_mcp_request, healthz, readyz]);

        if self.config.enable_cors {
            rocket = rocket.attach(Cors);
        }

        rocket
    }

    /// Start the HTTP transport server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        info!("HTTP transport listening on {}", addr);

        let figment = rocket::Config::figment()
            .merge(("address", self.config.host.clone()))
            .merge(("port", self.config.port));

        let rocket = self.rocket().configure(figment);

        rocket
            .launch()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    /// Start with graceful shutdown
    ///
    /// Note: Rocket handles graceful shutdown internally via Ctrl+C.
    pub async fn start_with_shutdown(
        self,
        _shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Rocket handles graceful shutdown internally
        self.start().await
    }
}

/// CORS Fairing for Rocket
///
/// Adds CORS headers to all responses to allow browser access.
pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    }
}

/// Handle MCP request via HTTP POST
///
/// Routes MCP JSON-RPC requests to the appropriate handlers based on method name.
///
/// # Supported Methods
///
/// - `initialize`: Returns server info and capabilities
/// - `tools/list`: Returns list of available tools
/// - `tools/call`: Executes a tool with provided arguments
/// - `ping`: Returns empty success response for health checks
#[post("/mcp", format = "json", data = "<request>")]
async fn handle_mcp_request(
    state: &State<HttpTransportState>,
    bridge_provenance: BridgeProvenance,
    request: Json<McpRequest>,
) -> Json<McpResponse> {
    let request = request.into_inner();
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(state, &request).await,
        "tools/list" => handle_tools_list(state, &request).await,
        "tools/call" => handle_tools_call(state, &bridge_provenance, &request).await,
        "ping" => McpResponse::success(request.id.clone(), serde_json::json!({})),
        _ => McpResponse::error(
            request.id.clone(),
            JSONRPC_METHOD_NOT_FOUND,
            format!("Unknown method: {}", request.method),
        ),
    };

    Json(response)
}

/// Handle the `initialize` method
///
/// Returns server information and capabilities.
async fn handle_initialize(state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    let server_info = state.server.get_info();

    let result = serde_json::json!({
        "protocolVersion": server_info.protocol_version.to_string(),
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": server_info.server_info.name,
            "version": server_info.server_info.version
        },
        "instructions": server_info.instructions
    });

    McpResponse::success(request.id.clone(), result)
}

/// Handle the `tools/list` method
///
/// Returns all available tools with their schemas.
async fn handle_tools_list(_state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    match create_tool_list() {
        Ok(tools) => {
            let tools_json: Vec<serde_json::Value> = tools
                .into_iter()
                .map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": serde_json::to_value(tool.input_schema.as_ref()).ok()
                    })
                })
                .collect();

            McpResponse::success(
                request.id.clone(),
                serde_json::json!({ "tools": tools_json }),
            )
        }
        Err(e) => {
            error!(error = ?e, "Failed to list tools");
            McpResponse::error(
                request.id.clone(),
                JSONRPC_INTERNAL_ERROR,
                format!("Failed to list tools: {:?}", e),
            )
        }
    }
}

/// Parse tool call parameters from the request
fn parse_tool_call_params(
    params: &serde_json::Value,
) -> Result<CallToolRequestParams, (i32, &'static str)> {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or((
            JSONRPC_INVALID_PARAMS,
            "Missing 'name' parameter for tools/call",
        ))?
        .to_string();

    let arguments = match params.get("arguments") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            let object = value.as_object().cloned().ok_or((
                JSONRPC_INVALID_PARAMS,
                "Invalid 'arguments' parameter for tools/call: expected object",
            ))?;
            Some(object)
        }
    };

    Ok(CallToolRequestParams {
        name: tool_name.into(),
        arguments,
        task: None,
        meta: None, // Meta is optional in MCP 2024-11-05+
    })
}

/// Convert tool call result to JSON response
fn tool_result_to_json(result: rmcp::model::CallToolResult) -> serde_json::Value {
    let content_json: Vec<serde_json::Value> = result
        .content
        .iter()
        .map(|content| match serde_json::to_value(content) {
            Ok(value) => value,
            Err(e) => serde_json::json!({
                "type": "text",
                "text": format!("Error serializing content: {}", e)
            }),
        })
        .collect();

    serde_json::json!({
        "content": content_json,
        "isError": result.is_error.unwrap_or(false)
    })
}

/// Handle the `tools/call` method
///
/// Executes the specified tool with the provided arguments.
async fn handle_tools_call(
    state: &HttpTransportState,
    bridge_provenance: &BridgeProvenance,
    request: &McpRequest,
) -> McpResponse {
    let has_workspace_provenance = bridge_provenance
        .workspace_root
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || bridge_provenance
            .repo_path
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());

    if !has_workspace_provenance {
        return McpResponse::error(
            request.id.clone(),
            JSONRPC_INVALID_PARAMS,
            "Direct HTTP tools/call is not supported. Use stdio or stdio bridge and provide workspace provenance headers.",
        );
    }

    let params = match &request.params {
        Some(params) => params,
        None => {
            return McpResponse::error(
                request.id.clone(),
                JSONRPC_INVALID_PARAMS,
                "Missing params for tools/call",
            );
        }
    };

    let mut call_request = match parse_tool_call_params(params) {
        Ok(req) => req,
        Err((code, msg)) => return McpResponse::error(request.id.clone(), code, msg),
    };

    let execution_context = ToolExecutionContext {
        session_id: bridge_provenance.session_id.clone(),
        parent_session_id: bridge_provenance.parent_session_id.clone(),
        project_id: bridge_provenance.project_id.clone(),
        worktree_id: bridge_provenance.worktree_id.clone(),
        repo_id: bridge_provenance.repo_id.clone(),
        repo_path: bridge_provenance
            .repo_path
            .clone()
            .or_else(|| bridge_provenance.workspace_root.clone()),
        operator_id: bridge_provenance.operator_id.clone(),
        machine_id: bridge_provenance.machine_id.clone(),
        agent_program: bridge_provenance.agent_program.clone(),
        model_id: bridge_provenance.model_id.clone(),
        delegated: bridge_provenance
            .delegated
            .as_deref()
            .map(str::trim)
            .and_then(|v| match v.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            }),
        timestamp: Some(chrono::Utc::now().timestamp()),
    };

    execution_context.apply_to_request_if_missing(&mut call_request);

    let handlers = ToolHandlers {
        index: state.server.index_handler(),
        search: state.server.search_handler(),
        validate: state.server.validate_handler(),
        memory: state.server.memory_handler(),
        session: state.server.session_handler(),
        agent: state.server.agent_handler(),
        project: state.server.project_handler(),
        vcs: state.server.vcs_handler(),
        vcs_entity: state.server.vcs_entity_handler(),
        plan_entity: state.server.plan_entity_handler(),
        issue_entity: state.server.issue_entity_handler(),
        org_entity: state.server.org_entity_handler(),
        entity: state.server.entity_handler(),
        hook_processor: state.server.hook_processor(),
    };

    match route_tool_call(call_request, &handlers, execution_context).await {
        Ok(result) => McpResponse::success(request.id.clone(), tool_result_to_json(result)),
        Err(e) => {
            error!(error = ?e, "Tool call failed");
            let code = if e.code.0 == JSONRPC_INVALID_PARAMS {
                JSONRPC_INVALID_PARAMS
            } else {
                JSONRPC_INTERNAL_ERROR
            };
            McpResponse::error(
                request.id.clone(),
                code,
                format!("Tool call failed: {:?}", e),
            )
        }
    }
}

// =============================================================================
// Health Endpoints
// =============================================================================

/// Liveness probe - returns 200 OK if the server is running
///
/// Used by Kubernetes/container orchestrators to check if the process is alive.
/// Always returns OK since if this responds, the process is running.
#[get("/healthz")]
fn healthz() -> &'static str {
    "OK"
}

/// Readiness probe - returns 200 OK if the server is ready to serve traffic
///
/// Used by Kubernetes/container orchestrators to check if the server can
/// handle requests. Currently returns OK if the MCP server is available.
#[get("/readyz")]
fn readyz(_state: &State<HttpTransportState>) -> &'static str {
    // Returns OK if server is running. Provider health checks are available
    // via the /health endpoint which returns detailed status JSON.
    "OK"
}
