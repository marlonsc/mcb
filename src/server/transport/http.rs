//! HTTP Transport for MCP
//!
//! Implements Streamable HTTP transport per MCP specification:
//! - POST /mcp: Client-to-server messages
//! - GET /mcp: Server-to-client SSE stream
//! - DELETE /mcp: Session termination

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, info};

use super::super::McpServer;
use super::config::TransportConfig;
use super::session::{SessionManager, SessionState};
use super::versioning::{headers, CompatibilityResult, VersionChecker};
use crate::infrastructure::connection_tracker::ConnectionTracker;
use crate::server::args::{
    ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::ServerHandler;

/// HTTP transport state shared across handlers
#[derive(Clone)]
pub struct HttpTransportState {
    pub server: Arc<McpServer>,
    pub session_manager: Arc<SessionManager>,
    pub version_checker: Arc<VersionChecker>,
    pub connection_tracker: Arc<ConnectionTracker>,
    pub config: TransportConfig,
}

/// Create the MCP HTTP transport router
pub fn create_mcp_router(state: HttpTransportState) -> Router {
    Router::new()
        // MCP message endpoint
        .route("/mcp", post(handle_mcp_post))
        .route("/mcp", get(handle_mcp_get))
        .route("/mcp", delete(handle_mcp_delete))
        // Version and health endpoints
        .route("/mcp/version", get(handle_version_info))
        .route("/mcp/health", get(handle_transport_health))
        .with_state(state)
}

/// Handle POST requests (client-to-server messages)
async fn handle_mcp_post(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
    Json(request): Json<serde_json::Value>,
) -> Result<Response, McpError> {
    // Track the request
    let _guard = state
        .connection_tracker
        .request_start()
        .ok_or(McpError::ServerDraining)?;

    // Check version compatibility
    check_version_compatibility(&state, &headers)?;

    // Get or create session
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Handle initialization (no session ID yet)
    if is_initialize_request(&request) {
        return handle_initialize(&state, request).await;
    }

    // Validate session for non-init requests
    let session_id = session_id.ok_or(McpError::MissingSessionId)?;

    let session = state
        .session_manager
        .get_session(&session_id)
        .ok_or(McpError::SessionNotFound)?;

    if session.state == SessionState::Terminated {
        return Err(McpError::SessionTerminated);
    }

    // Update session activity
    state.session_manager.touch_session(&session_id);

    // Process the request
    let response = process_mcp_request(&state, &request).await?;

    // Buffer response for resumption
    let mut resp = Json(&response).into_response();
    if let Some(event_id) = state
        .session_manager
        .buffer_message(&session_id, response.clone())
    {
        if let Ok(header_value) = HeaderValue::from_str(&event_id) {
            resp.headers_mut().insert("Mcp-Event-Id", header_value);
        }
    }

    // Add version headers
    add_version_headers(&state, resp.headers_mut());

    Ok(resp)
}

/// Handle GET requests (SSE stream for server-to-client)
async fn handle_mcp_get(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> Result<Response, McpError> {
    // Check version compatibility
    check_version_compatibility(&state, &headers)?;

    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or(McpError::MissingSessionId)?;

    let session = state
        .session_manager
        .get_session(session_id)
        .ok_or(McpError::SessionNotFound)?;

    if session.state == SessionState::Terminated {
        return Err(McpError::SessionTerminated);
    }

    // Server-Sent Events (SSE) streaming for server-to-client messages
    //
    // DECISION: Deferred to v0.2.0 (see docs/adr/011-http-transport-request-response-pattern.md)
    //
    // v0.1.0 uses request-response pattern (POST /mcp) which is:
    // - Functionally complete for all core operations
    // - Simpler and more reliable than SSE streaming
    // - Easier to test and debug
    //
    // Infrastructure for SSE is already in place:
    // - Session management and message buffering
    // - Event ID tracking for resumption
    // - No breaking changes needed when SSE is added
    //
    // v0.2.0 will implement SSE for real-time server-pushed updates.
    // In the meantime, clients can:
    // 1. Use POST request-response pattern for operations
    // 2. Implement polling for continuous updates if needed
    // 3. Use event bus pub/sub for async notifications
    Err(McpError::NotImplemented(
        "Server-Sent Events (SSE) streaming is not yet implemented in v0.1.0. \
         Use POST /mcp for request-response communication. \
         SSE streaming is planned for v0.2.0. \
         See docs/adr/011-http-transport-request-response-pattern.md for details."
            .to_string(),
    ))
}

/// Handle DELETE requests (session termination)
async fn handle_mcp_delete(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> Result<Response, McpError> {
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or(McpError::MissingSessionId)?;

    if state.session_manager.terminate_session(session_id) {
        info!("Terminated session: {}", session_id);
        let mut resp = StatusCode::NO_CONTENT.into_response();
        add_version_headers(&state, resp.headers_mut());
        Ok(resp)
    } else {
        Err(McpError::SessionNotFound)
    }
}

/// Handle version information requests
async fn handle_version_info(State(state): State<HttpTransportState>) -> Json<VersionInfoResponse> {
    Json(VersionInfoResponse {
        server: "MCP Context Browser".to_string(),
        version: state.version_checker.version_string(),
        protocol: state.version_checker.get_version_info(),
    })
}

/// Handle transport health check
async fn handle_transport_health(
    State(state): State<HttpTransportState>,
) -> Json<TransportHealthResponse> {
    Json(TransportHealthResponse {
        status: "healthy".to_string(),
        transport: "http".to_string(),
        active_sessions: state.session_manager.active_session_count(),
        total_sessions: state.session_manager.total_session_count(),
        active_requests: state.connection_tracker.active_count(),
        draining: state.connection_tracker.is_draining(),
    })
}

// Helpers

fn is_initialize_request(request: &serde_json::Value) -> bool {
    request.get("method").and_then(|v| v.as_str()) == Some("initialize")
}

fn check_version_compatibility(
    state: &HttpTransportState,
    headers: &HeaderMap,
) -> Result<(), McpError> {
    if let Some(expected) = headers
        .get(headers::EXPECTED_SERVER_VERSION)
        .and_then(|v| v.to_str().ok())
    {
        match state.version_checker.check_compatibility(expected) {
            CompatibilityResult::Compatible => Ok(()),
            CompatibilityResult::Warning { message } => {
                debug!("Version warning: {}", message);
                Ok(())
            }
            CompatibilityResult::Incompatible { message } => {
                Err(McpError::VersionIncompatible(message))
            }
        }
    } else {
        Ok(())
    }
}

fn add_version_headers(state: &HttpTransportState, headers: &mut axum::http::HeaderMap) {
    if let Ok(v) = HeaderValue::from_str(&state.version_checker.version_string()) {
        headers.insert(headers::SERVER_VERSION, v);
    }
}

async fn handle_initialize(
    state: &HttpTransportState,
    request: serde_json::Value,
) -> Result<Response, McpError> {
    // Create new session
    let session = state
        .session_manager
        .create_session()
        .map_err(|e| McpError::SessionError(e.to_string()))?;

    // Extract and store client info
    if let Some(client_info) = request.get("params").and_then(|p| p.get("clientInfo")) {
        state
            .session_manager
            .set_client_info(&session.id, client_info.clone());
    }

    // Activate the session
    state
        .session_manager
        .activate_session(&session.id)
        .map_err(|e| McpError::SessionError(e.to_string()))?;

    // Process initialization
    let response = process_mcp_request(state, &request).await?;

    // Build response with session ID
    let mut resp = Json(&response).into_response();
    if let Ok(header_value) = HeaderValue::from_str(&session.id) {
        resp.headers_mut().insert("Mcp-Session-Id", header_value);
    }

    // Add version headers
    add_version_headers(state, resp.headers_mut());

    info!("Created new session: {}", session.id);
    Ok(resp)
}

async fn process_mcp_request(
    state: &HttpTransportState,
    request: &serde_json::Value,
) -> Result<serde_json::Value, McpError> {
    debug!("Processing MCP request: {:?}", request.get("method"));

    let method = request
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing method".to_string()))?;

    let id = request.get("id").cloned();

    match method {
        "initialize" => {
            let info = state.server.get_info();
            Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": info.protocol_version,
                    "capabilities": info.capabilities,
                    "serverInfo": info.server_info,
                    "instructions": info.instructions
                }
            }))
        }
        "tools/list" => Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "index_codebase",
                        "description": "Index a codebase for semantic search",
                        "inputSchema": {}
                    },
                    {
                        "name": "search_code",
                        "description": "Search for code using natural language",
                        "inputSchema": {}
                    },
                    {
                        "name": "get_indexing_status",
                        "description": "Get current indexing status",
                        "inputSchema": {}
                    },
                    {
                        "name": "clear_index",
                        "description": "Clear the search index",
                        "inputSchema": {}
                    }
                ]
            }
        })),
        "tools/call" => {
            let params_val = request
                .get("params")
                .ok_or_else(|| McpError::InvalidRequest("Missing params".to_string()))?;

            let tool_name = params_val
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::InvalidRequest("Missing tool name".to_string()))?;

            let arguments = params_val
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(Default::default()));

            let result = match tool_name {
                "index_codebase" => {
                    let args: IndexCodebaseArgs =
                        serde_json::from_value(arguments).map_err(|e| {
                            McpError::InvalidRequest(format!("Invalid arguments: {}", e))
                        })?;
                    state.server.index_codebase(Parameters(args)).await
                }
                "search_code" => {
                    let args: SearchCodeArgs = serde_json::from_value(arguments).map_err(|e| {
                        McpError::InvalidRequest(format!("Invalid arguments: {}", e))
                    })?;
                    state.server.search_code(Parameters(args)).await
                }
                "get_indexing_status" => {
                    let args: GetIndexingStatusArgs =
                        serde_json::from_value(arguments).map_err(|e| {
                            McpError::InvalidRequest(format!("Invalid arguments: {}", e))
                        })?;
                    state
                        .server
                        .get_indexing_status_tool(Parameters(args))
                        .await
                }
                "clear_index" => {
                    let args: ClearIndexArgs = serde_json::from_value(arguments).map_err(|e| {
                        McpError::InvalidRequest(format!("Invalid arguments: {}", e))
                    })?;
                    state.server.clear_index(Parameters(args)).await
                }
                _ => {
                    return Err(McpError::InvalidRequest(format!(
                        "Unknown tool: {}",
                        tool_name
                    )))
                }
            }
            .map_err(|e| McpError::InternalError(e.to_string()))?;

            Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }))
        }
        _ => Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        })),
    }
}

// Response types

#[derive(Serialize)]
struct VersionInfoResponse {
    server: String,
    version: String,
    protocol: crate::server::transport::versioning::VersionInfo,
}

#[derive(Serialize)]
struct TransportHealthResponse {
    status: String,
    transport: String,
    active_sessions: usize,
    total_sessions: usize,
    active_requests: usize,
    draining: bool,
}

// Error handling

#[derive(Debug)]
pub enum McpError {
    MissingSessionId,
    SessionNotFound,
    SessionTerminated,
    SessionError(String),
    ServerDraining,
    VersionIncompatible(String),
    ProcessingError(String),
    InvalidRequest(String),
    InternalError(String),
    NotImplemented(String),
}

impl IntoResponse for McpError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match &self {
            McpError::MissingSessionId => (
                StatusCode::BAD_REQUEST,
                "Missing Mcp-Session-Id header".to_string(),
            ),
            McpError::SessionNotFound => (
                StatusCode::NOT_FOUND,
                "Session not found or expired".to_string(),
            ),
            McpError::SessionTerminated => {
                (StatusCode::GONE, "Session has been terminated".to_string())
            }
            McpError::SessionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            McpError::ServerDraining => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Server is shutting down".to_string(),
            ),
            McpError::VersionIncompatible(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            McpError::ProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            McpError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            McpError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            McpError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, msg.clone()),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}
