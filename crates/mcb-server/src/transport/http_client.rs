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

use mcb_domain::utils::mask_id;
use mcb_domain::value_objects::ids::SessionId;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::types::{McpRequest, McpResponse};
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_PARSE_ERROR};

/// MCP client transport configuration
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    /// Server URL (e.g., "http://127.0.0.1:8080")
    pub server_url: String,

    /// Session ID for this client connection
    pub session_id: SessionId,

    /// Request timeout
    pub timeout: Duration,
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
    /// Create a new HTTP client transport
    ///
    /// # Arguments
    ///
    /// * `server_url` - URL of the MCB server (e.g., "http://127.0.0.1:8080")
    /// * `session_prefix` - Optional prefix for session ID generation
    /// * `timeout` - Request timeout duration
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP client cannot be created.
    pub fn new(
        server_url: String,
        session_prefix: Option<String>,
        timeout: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let env_session_id = std::env::var("MCB_SESSION_ID").ok();
        let env_session_file = std::env::var("MCB_SESSION_FILE").ok();
        Self::new_with_session_source(
            server_url,
            session_prefix,
            timeout,
            env_session_id,
            env_session_file,
        )
    }

    /// Create a new HTTP client transport with explicit session source values.
    ///
    /// Used by tests to validate session source precedence without mutating process env.
    pub fn new_with_session_source(
        server_url: String,
        session_prefix: Option<String>,
        timeout: Duration,
        session_id_override: Option<String>,
        session_file_override: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let session_id =
            Self::resolve_session_id(session_prefix, session_id_override, session_file_override)?;

        let config = McpClientConfig {
            server_url,
            session_id,
            timeout,
        };

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self { config, client })
    }

    fn resolve_session_id(
        session_prefix: Option<String>,
        session_id_override: Option<String>,
        session_file_override: Option<String>,
    ) -> Result<SessionId, Box<dyn std::error::Error>> {
        if let Some(session_id) = session_id_override.and_then(Self::normalize_env_value) {
            return Ok(SessionId::new(session_id));
        }

        if let Some(session_file) = session_file_override.and_then(Self::normalize_env_value) {
            let path = Path::new(&session_file);

            if path.exists() {
                let existing = std::fs::read_to_string(path)?;
                if let Some(existing_id) = Self::normalize_env_value(existing) {
                    return Ok(SessionId::new(existing_id));
                }
            }

            let generated = Self::generate_session_id(session_prefix).into_string();
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, format!("{}\n", generated))?;
            return Ok(SessionId::new(generated));
        }

        Ok(Self::generate_session_id(session_prefix))
    }

    fn generate_session_id(session_prefix: Option<String>) -> SessionId {
        match session_prefix {
            Some(prefix) => SessionId::new(format!("{}_{}", prefix, Uuid::new_v4())),
            None => SessionId::new(Uuid::new_v4().to_string()),
        }
    }

    fn normalize_env_value(value: impl AsRef<str>) -> Option<String> {
        let normalized = value.as_ref().trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
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
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            server_url = %self.config.server_url,
            session_id = %mask_id(self.config.session_id.as_str()),
            "MCB client transport started"
        );

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        info!("stdin closed, shutting down");
                        break;
                    }
                    error!(error = %e, "Error reading from stdin");
                    continue;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            debug!(request = %line, "Received request from stdin");

            // Parse the request
            let request: McpRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    warn!(error = %e, line = %line, "Failed to parse request");
                    let error_response = Self::create_parse_error(e);
                    Self::write_response(&mut stdout, &error_response)?;
                    continue;
                }
            };

            // Forward to server and handle response
            let response = self.forward_request(&request).await;
            Self::write_response(&mut stdout, &response)?;
        }

        info!("MCB client transport finished");
        Ok(())
    }

    /// Send a request to the MCB server
    async fn send_request(&self, request: &McpRequest) -> Result<McpResponse, reqwest::Error> {
        let url = format!("{}/mcp", self.config.server_url);

        debug!(
            url = %url,
            method = %request.method,
            session_id = %mask_id(self.config.session_id.as_str()),
            "Sending request to server"
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let status = response.status();
        debug!(status = %status, "Received response from server");

        if !status.is_success() {
            warn!(status = %status, "Server returned non-success status");
        }

        response.json::<McpResponse>().await
    }

    /// Get the session ID for this client
    pub fn session_id(&self) -> &str {
        self.config.session_id.as_str()
    }

    /// Get the server URL
    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }

    /// Forward a request to the server, handling errors
    async fn forward_request(&self, request: &McpRequest) -> McpResponse {
        match self.send_request(request).await {
            Ok(resp) => resp,
            Err(e) => {
                error!(error = %e, "Failed to send request to server");
                Self::create_server_error(e, request.id.clone())
            }
        }
    }

    /// Create a JSON-RPC parse error response
    fn create_parse_error(e: serde_json::Error) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(super::types::McpError {
                code: JSONRPC_PARSE_ERROR,
                message: format!("Parse error: {}", e),
            }),
            id: None,
        }
    }

    /// Create a JSON-RPC server error response
    fn create_server_error(e: reqwest::Error, id: Option<serde_json::Value>) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(super::types::McpError {
                code: JSONRPC_INTERNAL_ERROR,
                message: format!("Server communication error: {}", e),
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
        debug!(response = %response_json, "Sending response to stdout");
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::HttpClientTransport;
    use std::time::Duration;

    #[test]
    fn session_id_override_takes_precedence_over_file() {
        let client = HttpClientTransport::new_with_session_source(
            "http://127.0.0.1:18080".to_string(),
            Some("prefix".to_string()),
            Duration::from_secs(10),
            Some("explicit-session-id".to_string()),
            Some("/tmp/ignored-session-file".to_string()),
        )
        .expect("create client");

        assert_eq!(client.session_id(), "explicit-session-id");
    }

    #[test]
    fn session_id_persists_via_session_file() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let session_file = temp_dir.path().join("session.id");
        let session_file_str = session_file.to_string_lossy().to_string();

        let first = HttpClientTransport::new_with_session_source(
            "http://127.0.0.1:18080".to_string(),
            Some("persist".to_string()),
            Duration::from_secs(10),
            None,
            Some(session_file_str.clone()),
        )
        .expect("create first client");

        let second = HttpClientTransport::new_with_session_source(
            "http://127.0.0.1:18080".to_string(),
            Some("persist".to_string()),
            Duration::from_secs(10),
            None,
            Some(session_file_str),
        )
        .expect("create second client");

        assert_eq!(first.session_id(), second.session_id());
    }
}
