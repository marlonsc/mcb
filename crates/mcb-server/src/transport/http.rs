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
//! |--------|-------------|
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
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Consolidated Admin API into single port in v0.2.0.

use std::net::SocketAddr;
use std::sync::Arc;

use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Build, Request, Response, Rocket, State, get, post, routes};
use tracing::{error, info};

use super::types::{McpRequest, McpResponse};
use crate::McpServer;
use crate::admin::auth::AdminAuthConfig;
use crate::admin::browse_handlers::BrowseState;
use crate::admin::handlers::AdminState;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{ToolHandlers, create_tool_list, route_tool_call};
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

    /// Build the Rocket application with MCP and optional Admin routes
    pub fn rocket(&self) -> Rocket<Build> {
        use crate::admin::browse_handlers::{
            get_collection_tree, get_file_chunks, list_collection_files, list_collections,
        };
        use crate::admin::config_handlers::{get_config, reload_config, update_config_section};
        use crate::admin::handlers::{
            extended_health_check, get_cache_stats, get_jobs_status, get_metrics, health_check,
            list_browse_project_issues, list_browse_project_phases, list_browse_projects,
            liveness_check, readiness_check, shutdown,
        };
        use crate::admin::lifecycle_handlers::{
            list_services, restart_service, services_health, start_service, stop_service,
        };
        use crate::admin::sse::events_stream;
        use crate::admin::web::handlers::{
            browse_collection_page, browse_file_page, browse_page, browse_tree_page, config_page,
            dashboard, dashboard_ui, favicon, health_page, jobs_page, shared_js, theme_css,
        };

        let mut rocket = rocket::build()
            .manage(self.state.clone())
            .mount("/", routes![handle_mcp_request, healthz, readyz]);

        // Mount admin routes if admin state is provided
        // Note: /events and /metrics routes are provided by admin routes (events_stream, get_metrics)
        if let Some(ref admin_state) = self.admin_state {
            rocket = rocket
                .manage(admin_state.clone())
                .manage(
                    self.auth_config
                        .clone()
                        .unwrap_or_else(|| Arc::new(AdminAuthConfig::default())),
                )
                .mount(
                    "/",
                    routes![
                        health_check,
                        extended_health_check,
                        get_metrics,
                        get_jobs_status,
                        list_browse_projects,
                        list_browse_project_phases,
                        list_browse_project_issues,
                        readiness_check,
                        liveness_check,
                        shutdown,
                        get_config,
                        reload_config,
                        update_config_section,
                        list_services,
                        services_health,
                        start_service,
                        stop_service,
                        restart_service,
                        get_cache_stats,
                        events_stream,
                        dashboard,
                        dashboard_ui,
                        favicon,
                        config_page,
                        health_page,
                        jobs_page,
                        browse_page,
                        browse_collection_page,
                        browse_file_page,
                        browse_tree_page,
                        theme_css,
                        shared_js,
                    ],
                );

            // Add browse routes if BrowseState is available
            if let Some(ref browse) = self.browse_state {
                rocket = rocket.manage(browse.clone()).mount(
                    "/",
                    routes![
                        list_collections,
                        list_collection_files,
                        get_file_chunks,
                        get_collection_tree,
                    ],
                );
            }
        }

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
    request: Json<McpRequest>,
) -> Json<McpResponse> {
    let request = request.into_inner();
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(state, &request).await,
        "tools/list" => handle_tools_list(state, &request).await,
        "tools/call" => handle_tools_call(state, &request).await,
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
async fn handle_tools_call(state: &HttpTransportState, request: &McpRequest) -> McpResponse {
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

    let call_request = match parse_tool_call_params(params) {
        Ok(req) => req,
        Err((code, msg)) => return McpResponse::error(request.id.clone(), code, msg),
    };

    let handlers = ToolHandlers {
        index: state.server.index_handler(),
        search: state.server.search_handler(),
        validate: state.server.validate_handler(),
        memory: state.server.memory_handler(),
        session: state.server.session_handler(),
        agent: state.server.agent_handler(),
        project: state.server.project_handler(),
        vcs: state.server.vcs_handler(),
        hook_processor: state.server.hook_processor(),
    };

    match route_tool_call(call_request, &handlers).await {
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
