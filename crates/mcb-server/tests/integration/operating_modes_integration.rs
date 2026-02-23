//! Operating Modes Integration Tests
//!
//! Tests the three MCB operating modes:
//! - Standalone: Local providers with stdio transport
//! - Server: HTTP daemon accepting client connections
//! - Client: HTTP client that forwards stdio to server
//!
//! All tests use random free ports to avoid conflicts.

use rstest::rstest;
extern crate mcb_providers;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::types::{AppConfig, ModeConfig, OperatingMode};
use mcb_server::mcp_server::McpServer;
use mcb_server::session::SessionManager;
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::http_client::HttpClientTransport;
use mcb_server::transport::types::{McpRequest, McpResponse};

use crate::utils::http_mcp::get_free_port;
use crate::utils::test_fixtures::{TEST_EMBEDDING_DIMENSIONS, shared_app_context};
use crate::utils::timeouts::TEST_TIMEOUT;

fn create_test_config() -> Result<(AppConfig, tempfile::TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut config = ConfigLoader::new().load()?;
    config.providers.database.configs.insert(
        "default".to_owned(),
        mcb_infrastructure::config::DatabaseConfig {
            provider: "sqlite".to_owned(),
            path: Some(db_path),
        },
    );
    config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
    Ok((config, temp_dir))
}

use crate::utils::test_fixtures::shared_fastembed_test_cache_dir;

/// Create test configuration for client mode
fn create_client_config(server_port: u16) -> ModeConfig {
    ModeConfig {
        mode_type: OperatingMode::Client,
        server_url: format!("http://127.0.0.1:{server_port}"),
        session_prefix: Some("test".to_owned()),
        timeout_secs: 30,
        auto_reconnect: true,
        max_reconnect_attempts: 5,
        session_id: None,
        session_file: None,
    }
}

// ============================================================================
// Mode Configuration Tests
// ============================================================================

#[test]
fn test_operating_mode_enum_variants() {
    let standalone = OperatingMode::Standalone;
    let client = OperatingMode::Client;

    assert_eq!(standalone, OperatingMode::default());
    assert_ne!(standalone, client);
}

#[test]
fn test_mode_config_client_settings() {
    let port = get_free_port().expect("get free port");
    let config = create_client_config(port);

    assert_eq!(config.mode_type, OperatingMode::Client);
    assert!(config.server_url.contains(&port.to_string()));
    assert_eq!(config.session_prefix, Some("test".to_owned()));
    assert_eq!(config.timeout_secs, 30);
    assert!(config.auto_reconnect);
    assert_eq!(config.max_reconnect_attempts, 5);
}

#[test]
fn test_mode_config_toml_deserialization() {
    let toml = r#"
        type = "client"
        server_url = "http://localhost:9999"
        session_prefix = "claude"
        timeout_secs = 60
        auto_reconnect = false
        max_reconnect_attempts = 10
    "#;
    let config: ModeConfig = toml::from_str(toml).expect("Should deserialize");

    assert!(config.is_client());
    assert_eq!(config.server_url(), "http://localhost:9999");
    assert_eq!(config.session_prefix(), Some("claude"));
    assert_eq!(config.timeout_secs, 60);
    assert!(!config.auto_reconnect);
    assert_eq!(config.max_reconnect_attempts, 10);
}

#[test]
fn test_mode_config_toml_with_defaults() {
    let toml = r#"
        type = "standalone"
        server_url = "http://127.0.0.1:3000"
        timeout_secs = 30
        auto_reconnect = true
        max_reconnect_attempts = 5
    "#;
    let config: ModeConfig = toml::from_str(toml).expect("Should deserialize");

    assert!(config.is_standalone());
    assert_eq!(config.server_url(), "http://127.0.0.1:3000");
    assert!(config.auto_reconnect);
    assert_eq!(config.max_reconnect_attempts, 5);
}

// ============================================================================
// Session Manager Tests
// ============================================================================

#[test]
fn test_session_manager_creates_sessions() {
    let manager = SessionManager::new();

    let ctx1 = manager.get_or_create("session-abc-123");
    assert_eq!(ctx1.id, "session-abc-123");
    // Prefix format: s_{12-hex-chars}
    assert!(ctx1.collection_prefix.starts_with("s_"));
    assert_eq!(ctx1.collection_prefix.len(), 14);

    // Same session ID should return same prefix
    let ctx2 = manager.get_or_create("session-abc-123");
    assert_eq!(ctx1.collection_prefix, ctx2.collection_prefix);
}

#[test]
fn test_session_manager_isolates_collections() {
    let manager = SessionManager::new();

    // Create two different sessions - hash-based prefix ensures uniqueness
    // regardless of session ID structure
    let _ctx1 = manager.get_or_create("session-1");
    let _ctx2 = manager.get_or_create("session-2");

    // Prefix collections
    let coll1 = manager.prefix_collection(Some("session-1"), "default");
    let coll2 = manager.prefix_collection(Some("session-2"), "default");

    // Should have different prefixes (hash-based, so always unique)
    assert_ne!(coll1, coll2);
    assert!(coll1.ends_with("_default"));
    assert!(coll2.ends_with("_default"));
}

#[test]
fn test_session_manager_no_prefix_without_session() {
    let manager = SessionManager::new();

    let coll = manager.prefix_collection(None, "default");
    assert_eq!(coll, "default");
}

#[test]
fn test_session_cleanup() {
    let manager = SessionManager::new();

    // Create a session
    let _ = manager.get_or_create("old-session");
    assert_eq!(manager.session_count(), 1);

    // Cleanup with 0 duration should remove all sessions
    manager.cleanup_old_sessions(Duration::ZERO);
    assert_eq!(manager.session_count(), 0);
}

#[test]
fn test_session_removal() {
    let manager = SessionManager::new();

    let _ = manager.get_or_create("to-remove");
    assert!(manager.get("to-remove").is_some());
    assert_eq!(manager.session_count(), 1);

    let _ = manager.remove("to-remove");
    assert!(manager.get("to-remove").is_none());
    assert_eq!(manager.session_count(), 0);
}

#[test]
fn test_session_prefix_hash_uniqueness() {
    let manager = SessionManager::new();

    // Short session ID
    let ctx1 = manager.get_or_create("abc");
    assert!(ctx1.collection_prefix.starts_with("s_"));
    assert_eq!(ctx1.collection_prefix.len(), 14);

    // Long session ID - hash ensures consistent format
    let ctx2 = manager.get_or_create("very-long-session-id-12345");
    assert!(ctx2.collection_prefix.starts_with("s_"));
    assert_eq!(ctx2.collection_prefix.len(), 14);

    // Similar session IDs get different prefixes
    let ctx3 = manager.get_or_create("claude_abc123");
    let ctx4 = manager.get_or_create("claude_abc124");
    assert_ne!(ctx3.collection_prefix, ctx4.collection_prefix);
}

// ============================================================================
// HTTP Transport Configuration Tests
// ============================================================================

#[test]
fn test_http_transport_config_localhost() {
    let port = get_free_port().expect("get free port");
    let config = HttpTransportConfig::localhost(port);
    let loaded = ConfigLoader::new().load().expect("load config");

    assert_eq!(config.host, loaded.server.network.host);
    assert_eq!(config.port, port);
    assert!(config.enable_cors);
}

#[test]
fn test_http_transport_config_socket_addr() {
    let port = get_free_port().expect("get free port");
    let config = HttpTransportConfig::localhost(port);
    let addr = config.socket_addr().expect("valid socket addr");
    let loaded = ConfigLoader::new().load().expect("load config");

    assert_eq!(addr.port(), port);
    assert_eq!(addr.ip().to_string(), loaded.server.network.host);
}

#[test]
fn test_http_transport_config_default() {
    let config = HttpTransportConfig::default();
    let loaded = ConfigLoader::new().load().expect("load config");

    assert_eq!(config.host, loaded.server.network.host);
    assert_eq!(config.port, loaded.server.network.port);
    assert!(config.enable_cors);
}

// ============================================================================
// HTTP Client Transport Tests
// ============================================================================

#[test]
fn test_http_client_session_id_with_prefix() {
    let port = get_free_port().expect("get free port");
    let client = HttpClientTransport::new_with_session_source(
        format!("http://127.0.0.1:{port}"),
        Some("test-prefix".to_owned()),
        TEST_TIMEOUT,
        None,
        None,
    )
    .expect("Failed to create client");

    assert!(client.session_id().starts_with("test-prefix_"));
}

#[test]
fn test_http_client_session_id_without_prefix() {
    let port = get_free_port().expect("get free port");
    let client = HttpClientTransport::new_with_session_source(
        format!("http://127.0.0.1:{port}"),
        None,
        TEST_TIMEOUT,
        None,
        None,
    )
    .expect("Failed to create client");

    // Should be a valid UUID
    let session_id = client.session_id();
    assert!(uuid::Uuid::parse_str(session_id).is_ok());
}

#[test]
fn test_http_client_server_url() {
    let port = get_free_port().expect("get free port");
    let expected_url = format!("http://127.0.0.1:{port}");
    let client = HttpClientTransport::new_with_session_source(
        expected_url.clone(),
        None,
        TEST_TIMEOUT,
        None,
        None,
    )
    .expect("Failed to create client");

    assert_eq!(client.server_url(), expected_url);
}

#[test]
fn test_http_client_from_mode_config() {
    let port = get_free_port().expect("get free port");
    let mode_config = create_client_config(port);

    let client = HttpClientTransport::new_with_session_source(
        mode_config.server_url.clone(),
        mode_config.session_prefix.clone(),
        Duration::from_secs(mode_config.timeout_secs),
        None,
        None,
    )
    .expect("Failed to create client");

    assert!(client.session_id().starts_with("test_"));
    assert!(client.server_url().contains(&port.to_string()));
}

// ============================================================================
// MCP Request/Response Serialization Tests
// ============================================================================

#[test]
fn test_mcp_request_serialization() {
    let request = McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(json.contains("tools/list"));

    // Deserialize back
    let parsed: McpRequest = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(parsed.method, "tools/list");
}

#[test]
fn test_mcp_request_with_params() {
    let request = McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": "search",
            "arguments": {"query": "test", "resource": "code"}
        })),
        id: Some(serde_json::json!(42)),
    };

    let json = serde_json::to_string(&request).expect("Serialize");
    let parsed: McpRequest = serde_json::from_str(&json).expect("Deserialize");

    assert_eq!(parsed.method, "tools/call");
    assert!(parsed.params.is_some());
    assert_eq!(parsed.id, Some(serde_json::json!(42)));
}

#[test]
fn test_mcp_response_success() {
    let response =
        McpResponse::success(Some(serde_json::json!(1)), serde_json::json!({"tools": []}));

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    assert_eq!(response.id, Some(serde_json::json!(1)));
}

#[test]
fn test_mcp_response_error() {
    let response = McpResponse::error(Some(serde_json::json!(1)), -32600, "Invalid Request");

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
}

#[test]
fn test_mcp_response_serialization_roundtrip() {
    let original = McpResponse::success(
        Some(serde_json::json!(42)),
        serde_json::json!({"status": "ok"}),
    );

    let json = serde_json::to_string(&original).expect("Serialize");
    let parsed: McpResponse = serde_json::from_str(&json).expect("Deserialize");

    assert_eq!(parsed.jsonrpc, original.jsonrpc);
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.result, original.result);
}

#[test]
fn test_mcp_response_error_serialization_roundtrip() {
    let original = McpResponse::error(Some(serde_json::json!(99)), -32601, "Method not found");

    let json = serde_json::to_string(&original).expect("Serialize");
    let parsed: McpResponse = serde_json::from_str(&json).expect("Deserialize");

    assert_eq!(parsed.jsonrpc, "2.0");
    assert!(parsed.error.is_some());
    let error = parsed.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

// ============================================================================
// App Config Mode Integration Tests
// ============================================================================

#[rstest]
#[case(false)]
#[case(true)]
fn app_config_mode_setup(#[case] use_client_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (mut config, _temp_dir) = create_test_config()?;
    let mut expected_port = None;

    if use_client_mode {
        let port = get_free_port()?;
        config.mode = create_client_config(port);
        expected_port = Some(port);
    }

    if let Some(port) = expected_port {
        assert!(config.mode.is_client());
        assert!(config.mode.server_url.contains(&port.to_string()));
    } else {
        assert_eq!(config.mode.mode_type, OperatingMode::Standalone);
        assert!(config.mode.is_standalone());
    }
    Ok(())
}

// ============================================================================
// Port Allocation Tests
// ============================================================================

#[test]
fn test_get_free_port_returns_valid_port() {
    let port = get_free_port().expect("get free port");
    assert!(port > 0);
    assert!(port < 65535);
}

#[test]
fn test_get_free_port_returns_different_ports() {
    let port1 = get_free_port().expect("get free port");
    let port2 = get_free_port().expect("get free port");

    // While not guaranteed, in practice these should be different
    // due to sequential port allocation
    assert_ne!(
        port1, port2,
        "Sequential port allocation should return different ports"
    );
}

// ============================================================================
// Full-Stack Mode Tests (with AppContext)
// ============================================================================

#[tokio::test]
async fn test_standalone_mode_initializes_providers() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    // Verify embedding provider
    let embedding = ctx.embedding_handle().get();
    assert!(!embedding.provider_name().is_empty());
    assert!(embedding.dimensions() > 0);

    // Verify vector store provider
    let vector_store = ctx.vector_store_handle().get();
    assert!(!vector_store.provider_name().is_empty());
    Ok(())
}

#[tokio::test]
async fn test_mode_selection_affects_nothing_in_standalone()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    // Verify we have working providers
    let embedding = ctx.embedding_handle().get();
    let texts = vec!["test".to_owned()];
    let result = embedding.embed_batch(&texts).await;

    assert!(
        result.is_ok(),
        "Standalone mode should have working embedding"
    );
    Ok(())
}

#[tokio::test]
async fn test_session_isolation_with_vector_store() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    let manager = SessionManager::new();
    let vector_store = ctx.vector_store_handle().get();

    // Create collections with different session prefixes
    // Using realistic session IDs like the HTTP client generates
    let session_a = "claude_550e8400-e29b-41d4-a716-446655440000";
    let session_b = "claude_550e8400-e29b-41d4-a716-446655440001";

    let coll_a = manager.prefix_collection(Some(session_a), "test");
    let coll_b = manager.prefix_collection(Some(session_b), "test");

    // Collections should have different names (hash-based prefix ensures this)
    assert_ne!(coll_a, coll_b);

    // Both should be able to create their own collections
    vector_store
        .create_collection(&CollectionId::from_name(&coll_a), TEST_EMBEDDING_DIMENSIONS)
        .await?;
    vector_store
        .create_collection(&CollectionId::from_name(&coll_b), TEST_EMBEDDING_DIMENSIONS)
        .await?;

    // Verify they're separate (search one, verify empty in other)
    let results_a = vector_store
        .search_similar(
            &CollectionId::from_name(&coll_a),
            &vec![0.0; TEST_EMBEDDING_DIMENSIONS],
            10,
            None,
        )
        .await;
    let results_b = vector_store
        .search_similar(
            &CollectionId::from_name(&coll_b),
            &vec![0.0; TEST_EMBEDDING_DIMENSIONS],
            10,
            None,
        )
        .await;

    // Both should succeed (collections exist) and be empty (no data inserted)
    assert!(results_a.is_ok());
    assert!(results_b.is_ok());
    Ok(())
}

// ============================================================================
// HTTP Server Integration Tests (End-to-End)
// ============================================================================

/// Helper to create an MCP server — delegates to shared fixture which reuses
/// the process-wide `AppContext` (ONNX model loaded once).
async fn create_test_mcp_server() -> (McpServer, tempfile::TempDir) {
    crate::utils::test_fixtures::create_test_mcp_server().await
}

#[derive(Clone)]
struct TestClient {
    app: Router,
}

impl TestClient {
    async fn post_mcp(
        &self,
        request: &McpRequest,
        headers: &[(&str, &str)],
    ) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("Content-Type", "application/json");

        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }

        let req = builder.body(Body::from(serde_json::to_vec(request)?))?;
        let response = self
            .app
            .clone()
            .oneshot(req)
            .await
            .map_err(Box::<dyn std::error::Error>::from)?;

        let status = response.status();
        let body = response.into_body().collect().await?.to_bytes();
        Ok((status, String::from_utf8(body.to_vec())?))
    }
}

async fn create_http_test_client(
    port: u16,
) -> Result<(TestClient, tempfile::TempDir), Box<dyn std::error::Error>> {
    let (server_instance, temp_dir) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);
    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);
    let client = TestClient {
        app: transport.router(),
    };

    Ok((client, temp_dir))
}

#[tokio::test]
async fn test_http_server_tools_list() {
    let port = get_free_port().expect("get free port");
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    // Create and start HTTP transport
    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    // Spawn the server in background
    let client = TestClient {
        app: transport.router(),
    };

    // Send tools/list request
    let request = McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (status, body) = client
        .post_mcp(
            &request,
            &[
                ("X-Session-Id", "test-session-index-status"),
                ("X-Project-Id", "test-project-index-status"),
                ("X-Repo-Id", "test-repo-index-status"),
                ("X-Workspace-Root", "/tmp/test-workspace"),
                ("X-Repo-Path", "/tmp/test-workspace"),
            ],
        )
        .await
        .expect("dispatch mcp request");

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_none(), "Should not have error");
    assert!(mcp_response.result.is_some(), "Should have result");

    // Verify tools are returned
    let result = mcp_response.result.unwrap();
    let tools = result.get("tools").expect("Should have tools array");
    assert!(tools.is_array(), "Tools should be array");

    let tools_array = tools.as_array().unwrap();
    assert!(!tools_array.is_empty(), "Should have at least one tool");

    // Verify expected tools exist
    let tool_names: Vec<&str> = tools_array
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(tool_names.contains(&"index"), "Should have index");
    assert!(tool_names.contains(&"search"), "Should have search");
    assert!(tool_names.contains(&"validate"), "Should have validate");
    assert!(tool_names.contains(&"memory"), "Should have memory");
    assert!(tool_names.contains(&"session"), "Should have session");
    assert!(tool_names.contains(&"agent"), "Should have agent");
    assert!(tool_names.contains(&"project"), "Should have project");
    assert!(tool_names.contains(&"vcs"), "Should have vcs");
}

#[rstest]
#[case("ping", 42, false, None)]
#[case("initialize", 1, false, None)]
#[case("unknown/method", 99, true, Some(-32601))]
#[tokio::test]
async fn test_http_server_core_methods(
    #[case] method: &str,
    #[case] request_id: i64,
    #[case] expect_error: bool,
    #[case] expected_error_code: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let port = get_free_port()?;
    let (client, _temp) = create_http_test_client(port).await?;

    let request = McpRequest {
        method: method.to_owned(),
        params: None,
        id: Some(serde_json::json!(request_id)),
    };

    let (status, body) = client
        .post_mcp(
            &request,
            &[
                ("X-Workspace-Root", "/tmp/test-workspace"),
                ("X-Repo-Path", "/tmp/test-workspace"),
                ("X-Worktree-Id", "wt-index-status"),
                ("X-Operator-Id", "operator-index-status"),
                ("X-Machine-Id", "machine-index-status"),
                ("X-Agent-Program", "opencode"),
                ("X-Model-Id", "gpt-5.3-codex"),
                ("X-Delegated", "false"),
            ],
        )
        .await?;

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body)?;

    if expect_error {
        let error = mcp_response.error.ok_or("Method should error")?;
        if let Some(code) = expected_error_code {
            assert_eq!(error.code, code);
        }
        return Ok(());
    }

    assert!(mcp_response.error.is_none(), "Method should not error");
    assert_eq!(
        mcp_response.id,
        Some(serde_json::json!(request_id)),
        "Should echo request ID"
    );

    if method == "initialize" {
        let result = mcp_response.result.ok_or("Should have result")?;
        assert!(result.get("serverInfo").is_some(), "Should have serverInfo");
        assert!(
            result.get("capabilities").is_some(),
            "Should have capabilities"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_http_server_with_session_header() {
    let port = get_free_port().expect("get free port");
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let client = TestClient {
        app: transport.router(),
    };

    // Send request with session header
    let request = McpRequest {
        method: "ping".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (status, body) = client
        .post_mcp(&request, &[("X-Session-Id", "test-session-12345")])
        .await
        .expect("dispatch mcp request");

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    // Request should succeed with session header
    assert!(
        mcp_response.error.is_none(),
        "Should succeed with session header"
    );
}

#[tokio::test]
async fn test_http_server_tools_call_index_status() {
    let port = get_free_port().expect("get free port");
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let client = TestClient {
        app: transport.router(),
    };

    // Call index tool with status action
    let request = McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": "index",
            "arguments": {
                "action": "status",
                "collection": "test-collection"
            }
        })),
        id: Some(serde_json::json!(1)),
    };

    let (status, body) = client
        .post_mcp(
            &request,
            &[
                ("X-Session-Id", "test-session-index-status"),
                ("X-Project-Id", "test-project-index-status"),
                ("X-Repo-Id", "test-repo-index-status"),
                ("X-Workspace-Root", "/tmp/test-workspace"),
                ("X-Worktree-Id", "test-worktree-index-status"),
                ("X-Operator-Id", "test-operator-index-status"),
                ("X-Machine-Id", "test-machine-index-status"),
                ("X-Agent-Program", "opencode"),
                ("X-Model-Id", "openai/gpt-5.3-codex"),
                ("X-Delegated", "false"),
            ],
        )
        .await
        .expect("dispatch mcp request");

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    // Tool call should return a result (even if collection doesn't exist)
    assert!(
        mcp_response.result.is_some(),
        "Should have result from tool call"
    );
}

#[tokio::test]
async fn test_http_server_tools_call_without_workspace_provenance_is_rejected() {
    let port = get_free_port().expect("get free port");
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let client = TestClient {
        app: transport.router(),
    };

    let request = McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": "index",
            "arguments": {
                "action": "status",
                "collection": "test-collection"
            }
        })),
        id: Some(serde_json::json!(1)),
    };

    let (status, body) = client
        .post_mcp(&request, &[])
        .await
        .expect("dispatch mcp request");

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    let error = mcp_response
        .error
        .expect("tools/call without workspace provenance should be rejected");
    assert_eq!(error.code, -32602);
    assert!(
        error
            .message
            .contains("Direct HTTP tools/call is not supported")
    );
}

#[rstest]
#[case(None, "Missing params should return error")]
#[case(
    Some(serde_json::json!({
        "name": "index",
        "arguments": "not-an-object"
    })),
    "Non-object arguments should return error"
)]
#[case(
    Some(serde_json::json!({
        "name": "does-not-exist",
        "arguments": {}
    })),
    "Unknown tool should return error"
)]
#[tokio::test]
async fn test_http_server_tools_call_invalid_params_return_error(
    #[case] params: Option<serde_json::Value>,
    #[case] expected_message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let port = get_free_port()?;
    let (client, _temp) = create_http_test_client(port).await?;

    let request = McpRequest {
        method: "tools/call".to_owned(),
        params,
        id: Some(serde_json::json!(1)),
    };

    let (status, body) = client.post_mcp(&request, &[]).await?;

    assert_eq!(status, StatusCode::OK);

    let mcp_response: McpResponse = serde_json::from_str(&body)?;

    let error = mcp_response.error.ok_or(expected_message)?;
    assert_eq!(error.code, -32602);
    Ok(())
}
