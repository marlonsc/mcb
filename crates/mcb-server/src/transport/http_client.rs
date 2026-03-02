//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! HTTP Client Transport
//!
//! MCP client that connects to a remote MCB server via HTTP.
//! Reads MCP requests from stdin, forwards them to the server,
//! and writes responses to stdout.
//!
//! This enables MCB to run in "client mode" where it acts as a
//! stdio-to-HTTP bridge for Claude Code integration.

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Duration;

use hostname;
use mcb_domain::utils::id as domain_id;
use mcb_domain::utils::id::mask_id;
use mcb_domain::{debug, error, info, warn};

use super::types::{McpRequest, McpResponse};
use crate::constants::protocol::{
    CONTENT_TYPE_JSON, EXECUTION_FLOW_HYBRID, HTTP_HEADER_EXECUTION_FLOW, JSONRPC_VERSION,
    MCP_ENDPOINT_PATH,
};
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_PARSE_ERROR};

/// MCP client transport configuration
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    /// Server URL (e.g., "<http://127.0.0.1:8080>")
    pub server_url: String,

    /// Local client instance identifier for log correlation.
    pub client_instance_id: String,

    /// Public non-sensitive session identifier for local introspection/tests.
    pub public_session_id: String,

    /// Request timeout
    pub timeout: Duration,

    /// Workspace root path for provenance headers (auto-detected from CWD).
    pub workspace_root: Option<String>,

    /// Repository path for provenance headers.
    pub repo_path: Option<String>,
}

/// HTTP client transport
///
/// Bridges stdio (for Claude Code) to HTTP (for MCB server).
/// Each request is forwarded to the server over JSON-RPC.
pub struct HttpClientTransport {
    config: McpClientConfig,
    client: reqwest::Client,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport with explicit session source values.
    ///
    /// Used by tests to validate session source precedence without mutating process env.
    ///
    /// # Errors
    /// Returns an error when secure transport validation, session initialization, or client construction fails.
    pub fn new_with_session_source(
        server_url: String,
        session_prefix: Option<String>,
        timeout: Duration,
        session_id_override: Option<String>,
        session_file_override: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::require_secure_transport(&server_url)?;

        let public_session_id = Self::generate_session_id(session_prefix.clone());
        Self::initialize_session_state(session_prefix, session_id_override, session_file_override)?;

        let workspace_root = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from));

        let config = McpClientConfig {
            server_url,
            client_instance_id: domain_id::generate().to_string(),
            public_session_id,
            timeout,
            workspace_root: workspace_root.clone(),
            repo_path: workspace_root,
        };

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self { config, client })
    }

    fn initialize_session_state(
        session_prefix: Option<String>,
        session_id_override: Option<String>,
        session_file_override: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if session_id_override
            .and_then(Self::normalize_env_value)
            .is_some()
        {
            return Ok(());
        }

        if let Some(session_file) = session_file_override.and_then(Self::normalize_env_value) {
            let path = Path::new(&session_file);

            if path.exists() {
                let existing = std::fs::read_to_string(path)?;
                if Self::normalize_env_value(existing).is_some() {
                    return Ok(());
                }
            }

            let generated = Self::generate_session_id(session_prefix);
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, format!("{generated}\n"))?;
            return Ok(());
        }

        Ok(())
    }

    fn generate_session_id(session_prefix: Option<String>) -> String {
        match session_prefix {
            Some(prefix) => format!("{}_{}", prefix, domain_id::generate()),
            None => domain_id::generate().to_string(),
        }
    }

    /// Reject cleartext HTTP for non-loopback hosts.
    ///
    /// HTTPS is always accepted. Plain HTTP is only permitted when the host is
    /// a loopback address (`127.0.0.1`, `localhost`, `[::1]`), since the
    /// traffic never leaves the local machine. Any other combination is
    /// rejected to prevent cleartext transmission of sensitive data.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the URL uses plain HTTP with a non-loopback host.
    pub fn require_secure_transport(url: &str) -> Result<(), String> {
        let lower = url.to_ascii_lowercase();

        if lower.starts_with("https://") {
            return Ok(());
        }

        if let Some(after_scheme) = lower.strip_prefix("http://") {
            let host = if after_scheme.starts_with('[') {
                match after_scheme.find(']') {
                    Some(end) => &after_scheme[..=end],
                    None => after_scheme,
                }
            } else {
                after_scheme.split([':', '/']).next().unwrap_or("")
            };

            return match host {
                "127.0.0.1" | "localhost" | "[::1]" => Ok(()),
                _ => Err(format!(
                    "Cleartext HTTP is only allowed for loopback addresses \
                     (127.0.0.1, localhost, [::1]). \
                     Use HTTPS for remote host: {host}"
                )),
            };
        }

        Err(format!("Unsupported URL scheme in: {url}"))
    }

    fn normalize_env_value(value: impl AsRef<str>) -> Option<String> {
        let normalized = value.as_ref().trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_owned())
        }
    }

    /// Run the client transport
    ///
    /// Main loop that:
    /// 1. Reads JSON-RPC requests from stdin
    /// 2. Forwards them to the MCB server via HTTP
    /// 3. Writes responses to stdout
    ///
    /// Runs until stdin is closed (EOF).
    ///
    /// # Errors
    /// Returns an error when writing responses to stdout fails.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "HttpClient",
            "MCB client transport started",
            &format!(
                "server_url={} client_instance_id={}",
                self.config.server_url,
                mask_id(&self.config.client_instance_id)
            )
        );

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        info!("HttpClient", "stdin closed, shutting down");
                        break;
                    }
                    error!("HttpClient", "Error reading from stdin", &e);
                    continue;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            debug!("HttpClient", "Received request from stdin", &line.len());

            // Parse the request
            let request: McpRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    warn!(
                        "HttpClient",
                        "Failed to parse request",
                        &format!("error={e} len={}", line.len())
                    );
                    let error_response = Self::create_parse_error(&e);
                    Self::write_response(&mut stdout, &error_response)?;
                    continue;
                }
            };

            // Forward to server and handle response
            let response = self.forward_request(&request).await;
            Self::write_response(&mut stdout, &response)?;
        }

        info!("HttpClient", "MCB client transport finished");
        Ok(())
    }

    /// Send a request to the MCB server
    async fn send_request(&self, request: &McpRequest) -> Result<McpResponse, reqwest::Error> {
        let url = format!("{}{MCP_ENDPOINT_PATH}", self.config.server_url);

        debug!(
            "HttpClient",
            "Sending request to server",
            &format!(
                "url={url} method={} client_instance_id={}",
                request.method,
                mask_id(&self.config.client_instance_id)
            )
        );

        let response = post_mcp_request(&self.client, &url, request, &self.config).await?;

        let status = response.status();
        debug!("HttpClient", "Received response from server", &status);

        if !status.is_success() {
            warn!("HttpClient", "Server returned non-success status", &status);
        }

        response.json::<McpResponse>().await
    }

    /// Get the server URL
    #[must_use]
    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }

    /// Get the local public session identifier.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.config.public_session_id
    }

    /// Forward a request to the server, handling errors
    async fn forward_request(&self, request: &McpRequest) -> McpResponse {
        match self.send_request(request).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("HttpClient", "Failed to send request to server", &e);
                Self::create_server_error(&e, request.id.clone())
            }
        }
    }

    /// Create a JSON-RPC parse error response
    fn create_parse_error(e: &serde_json::Error) -> McpResponse {
        McpResponse {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: None,
            error: Some(super::types::McpError {
                code: JSONRPC_PARSE_ERROR,
                message: format!("Parse error: {e}"),
            }),
            id: None,
        }
    }

    /// Create a JSON-RPC server error response
    fn create_server_error(e: &reqwest::Error, id: Option<serde_json::Value>) -> McpResponse {
        McpResponse {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: None,
            error: Some(super::types::McpError {
                code: JSONRPC_INTERNAL_ERROR,
                message: format!("Server communication error: {e}"),
            }),
            id,
        }
    }

    /// Write a response to stdout
    fn write_response(
        stdout: &mut io::Stdout,
        response: &McpResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response_json = serde_json::to_string(response)?;
        debug!("HttpClient", "Sending response to stdout", &response_json);
        writeln!(stdout, "{response_json}")?;
        stdout.flush()?;
        Ok(())
    }
}

async fn post_mcp_request(
    client: &reqwest::Client,
    url: &str,
    request: &McpRequest,
    config: &McpClientConfig,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut builder = client
        .post(url)
        .header("Content-Type", CONTENT_TYPE_JSON)
        .header(HTTP_HEADER_EXECUTION_FLOW, EXECUTION_FLOW_HYBRID);

    if let Some(ref ws) = config.workspace_root {
        builder = builder.header("X-Workspace-Root", ws);
    }
    if let Some(ref rp) = config.repo_path {
        builder = builder.header("X-Repo-Path", rp);
    }
    builder = builder.header("X-Session-Id", &config.public_session_id);

    if let Ok(user) = std::env::var("USER") {
        builder = builder.header("X-Operator-Id", user);
    }

    let machine_id = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".to_owned());
    builder = builder.header("X-Machine-Id", machine_id);

    builder = builder.header("X-Agent-Program", "mcb-client");
    builder = builder.header("X-Model-Id", "unknown");
    builder = builder.header("X-Delegated", "false");

    builder.json(request).send().await
}
