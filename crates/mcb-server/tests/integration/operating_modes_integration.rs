//! Operating Modes Integration Tests
//!
//! Tests the three MCB operating modes:
//! - Standalone: Local providers with stdio transport
//! - Server: HTTP daemon accepting client connections
//! - Client: HTTP client that forwards stdio to server
//!
//! All tests use random free ports to avoid conflicts.

extern crate mcb_providers;

use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use crate::test_utils::mock_services::{
    TestIssueEntityRepository, TestOrgEntityRepository, TestPlanEntityRepository,
    TestProjectRepository, TestVcsEntityRepository, TestVcsProvider,
};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::types::{AppConfig, ModeConfig, OperatingMode};
use mcb_infrastructure::crypto::CryptoService;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_server::McpServerBuilder;
use mcb_server::mcp_server::McpServer;
use mcb_server::session::SessionManager;
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::http_client::HttpClientTransport;
use mcb_server::transport::types::{McpRequest, McpResponse};

/// Get a random available port by binding to port 0 and extracting the assigned port
fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    drop(listener);
    port
}

fn create_test_config() -> (AppConfig, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(db_path);
    (config, temp_dir)
}

/// Create test configuration for client mode
fn create_client_config(server_port: u16) -> ModeConfig {
    ModeConfig {
        mode_type: OperatingMode::Client,
        server_url: format!("http://127.0.0.1:{}", server_port),
        session_prefix: Some("test".to_string()),
        timeout_secs: 30,
        auto_reconnect: true,
        max_reconnect_attempts: 5,
    }
}

// ============================================================================
// Mode Configuration Tests
// ============================================================================

#[test]
fn test_mode_config_defaults_to_standalone() {
    let config = ModeConfig::default();
    assert_eq!(config.mode_type, OperatingMode::Standalone);
    assert!(config.is_standalone());
    assert!(!config.is_client());
}

#[test]
fn test_operating_mode_enum_variants() {
    let standalone = OperatingMode::Standalone;
    let client = OperatingMode::Client;

    assert_eq!(standalone, OperatingMode::default());
    assert_ne!(standalone, client);
}

#[test]
fn test_mode_config_client_settings() {
    let port = get_free_port();
    let config = create_client_config(port);

    assert_eq!(config.mode_type, OperatingMode::Client);
    assert!(config.server_url.contains(&port.to_string()));
    assert_eq!(config.session_prefix, Some("test".to_string()));
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
    "#;
    let config: ModeConfig = toml::from_str(toml).expect("Should deserialize");

    assert!(config.is_standalone());
    // Check defaults are applied
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
    manager.get_or_create("old-session");
    assert_eq!(manager.session_count(), 1);

    // Cleanup with 0 duration should remove all sessions
    manager.cleanup_old_sessions(Duration::ZERO);
    assert_eq!(manager.session_count(), 0);
}

#[test]
fn test_session_removal() {
    let manager = SessionManager::new();

    manager.get_or_create("to-remove");
    assert!(manager.get("to-remove").is_some());
    assert_eq!(manager.session_count(), 1);

    manager.remove("to-remove");
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
    let port = get_free_port();
    let config = HttpTransportConfig::localhost(port);
    let loaded = ConfigLoader::new().load().expect("load config");

    assert_eq!(config.host, loaded.server.network.host);
    assert_eq!(config.port, port);
    assert!(config.enable_cors);
}

#[test]
fn test_http_transport_config_socket_addr() {
    let port = get_free_port();
    let config = HttpTransportConfig::localhost(port);
    let addr = config.socket_addr();
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
    let port = get_free_port();
    let client = HttpClientTransport::new(
        format!("http://127.0.0.1:{}", port),
        Some("test-prefix".to_string()),
        Duration::from_secs(30),
    )
    .expect("Failed to create client");

    assert!(client.session_id().starts_with("test-prefix_"));
}

#[test]
fn test_http_client_session_id_without_prefix() {
    let port = get_free_port();
    let client = HttpClientTransport::new(
        format!("http://127.0.0.1:{}", port),
        None,
        Duration::from_secs(30),
    )
    .expect("Failed to create client");

    // Should be a valid UUID
    let session_id = client.session_id();
    assert!(uuid::Uuid::parse_str(session_id).is_ok());
}

#[test]
fn test_http_client_server_url() {
    let port = get_free_port();
    let expected_url = format!("http://127.0.0.1:{}", port);
    let client = HttpClientTransport::new(expected_url.clone(), None, Duration::from_secs(30))
        .expect("Failed to create client");

    assert_eq!(client.server_url(), expected_url);
}

#[test]
fn test_http_client_from_mode_config() {
    let port = get_free_port();
    let mode_config = create_client_config(port);

    let client = HttpClientTransport::new(
        mode_config.server_url.clone(),
        mode_config.session_prefix.clone(),
        Duration::from_secs(mode_config.timeout_secs),
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
        method: "tools/list".to_string(),
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
        method: "tools/call".to_string(),
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

#[test]
fn test_app_config_default_mode_is_standalone() {
    let (config, _temp_dir) = create_test_config();

    assert_eq!(config.mode.mode_type, OperatingMode::Standalone);
    assert!(config.mode.is_standalone());
}

#[test]
fn test_app_config_with_client_mode() {
    let (mut config, _temp_dir) = create_test_config();
    let port = get_free_port();

    config.mode = create_client_config(port);

    assert!(config.mode.is_client());
    assert!(config.mode.server_url.contains(&port.to_string()));
}

// ============================================================================
// Port Allocation Tests
// ============================================================================

#[test]
fn test_get_free_port_returns_valid_port() {
    let port = get_free_port();
    assert!(port > 0);
    assert!(port < 65535);
}

#[test]
fn test_get_free_port_returns_different_ports() {
    let port1 = get_free_port();
    let port2 = get_free_port();

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
async fn test_standalone_mode_initializes_providers() {
    let (config, _temp_dir) = create_test_config();

    // In standalone mode, init_app creates local providers
    let ctx = init_app(config).await.expect("Failed to init app");

    // Verify embedding provider
    let embedding = ctx.embedding_handle().get();
    assert!(!embedding.provider_name().is_empty());
    assert!(embedding.dimensions() > 0);

    // Verify vector store provider
    let vector_store = ctx.vector_store_handle().get();
    assert!(!vector_store.provider_name().is_empty());
}

#[tokio::test]
async fn test_mode_selection_affects_nothing_in_standalone() {
    // In standalone mode, we don't connect to any server
    // Everything runs locally
    let (mut config, _temp_dir) = create_test_config();
    config.mode.mode_type = OperatingMode::Standalone;

    let ctx = init_app(config).await.expect("Init should succeed");

    // Verify we have working providers
    let embedding = ctx.embedding_handle().get();
    let texts = vec!["test".to_string()];
    let result = embedding.embed_batch(&texts).await;

    assert!(
        result.is_ok(),
        "Standalone mode should have working embedding"
    );
}

#[tokio::test]
async fn test_session_isolation_with_vector_store() {
    let (config, _temp_dir) = create_test_config();
    let ctx = init_app(config).await.expect("Failed to init app");

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
        .create_collection(&CollectionId::new(&coll_a), 384)
        .await
        .expect("Create A");
    vector_store
        .create_collection(&CollectionId::new(&coll_b), 384)
        .await
        .expect("Create B");

    // Verify they're separate (search one, verify empty in other)
    let results_a = vector_store
        .search_similar(&CollectionId::new(&coll_a), &vec![0.0; 384], 10, None)
        .await;
    let results_b = vector_store
        .search_similar(&CollectionId::new(&coll_b), &vec![0.0; 384], 10, None)
        .await;

    // Both should succeed (collections exist) and be empty (no data inserted)
    assert!(results_a.is_ok());
    assert!(results_b.is_ok());
}

// ============================================================================
// HTTP Server Integration Tests (End-to-End)
// ============================================================================

/// Helper to create an MCP server with null providers for testing
async fn create_test_mcp_server() -> (McpServer, tempfile::TempDir) {
    let (config, _temp_dir) = create_test_config();
    let ctx = init_app(config.clone()).await.expect("Failed to init app");

    // Get providers from context
    let embedding_provider = ctx.embedding_handle().get();
    let vector_store_provider = ctx.vector_store_handle().get();
    let language_chunker = ctx.language_handle().get();
    let cache_provider = ctx.cache_handle().get();
    let indexing_ops = ctx.indexing();
    let event_bus = ctx.event_bus();

    // Create shared cache provider for domain services factory
    let shared_cache = SharedCacheProvider::from_arc(cache_provider);

    // Create crypto service with random key for tests
    let master_key = CryptoService::generate_master_key();
    let crypto = CryptoService::new(master_key).expect("Failed to create crypto service");

    // Use a temporary file for the memory database
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (memory_repository, shared_executor) =
        mcb_providers::database::create_memory_repository_with_executor(db_path)
            .await
            .expect("Failed to create memory database");
    let agent_repository = mcb_providers::database::create_agent_repository_from_executor(
        std::sync::Arc::clone(&shared_executor),
    );
    let vcs_provider: std::sync::Arc<dyn mcb_domain::ports::providers::VcsProvider> =
        std::sync::Arc::new(TestVcsProvider::new());

    let project_service: std::sync::Arc<dyn mcb_domain::ports::services::ProjectDetectorService> =
        std::sync::Arc::new(mcb_infrastructure::project::ProjectService::new());
    let project_repository = std::sync::Arc::new(TestProjectRepository::new());
    let file_hash_repository: std::sync::Arc<
        dyn mcb_domain::ports::repositories::FileHashRepository,
    > = std::sync::Arc::new(mcb_providers::database::SqliteFileHashRepository::new(
        std::sync::Arc::clone(&shared_executor),
        mcb_providers::database::SqliteFileHashConfig::default(),
    ));

    let deps = ServiceDependencies {
        project_id: "test-project".to_string(),
        cache: shared_cache,
        crypto: std::sync::Arc::new(crypto)
            as std::sync::Arc<dyn mcb_domain::ports::providers::CryptoProvider>,
        config,
        embedding_provider,
        vector_store_provider,
        language_chunker,
        indexing_ops,
        event_bus,
        memory_repository,
        agent_repository,
        file_hash_repository,
        vcs_provider,
        project_service,
        project_repository: project_repository.clone(),
        vcs_entity_repository: std::sync::Arc::new(TestVcsEntityRepository::new()),
        plan_entity_repository: std::sync::Arc::new(TestPlanEntityRepository::new()),
        issue_entity_repository: std::sync::Arc::new(TestIssueEntityRepository::new()),
        org_entity_repository: std::sync::Arc::new(TestOrgEntityRepository::new()),
    };

    let services = DomainServicesFactory::create_services(deps)
        .await
        .expect("Failed to create services");

    let server = McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_project_service(services.project_service)
        .with_project_workflow_service(services.project_repository)
        .with_vcs_provider(services.vcs_provider)
        .with_vcs_entity_repository(services.vcs_entity_repository)
        .with_plan_entity_repository(services.plan_entity_repository)
        .with_issue_entity_repository(services.issue_entity_repository)
        .with_org_entity_repository(services.org_entity_repository)
        .build()
        .expect("Failed to build MCP server");

    (server, temp_dir)
}

#[tokio::test]
async fn test_http_server_tools_list() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    // Create and start HTTP transport
    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    // Spawn the server in background
    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send tools/list request
    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Workspace-Root",
            "/tmp/test-workspace",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
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

#[tokio::test]
async fn test_http_server_ping() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send ping request
    let request = McpRequest {
        method: "ping".to_string(),
        params: None,
        id: Some(serde_json::json!(42)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Workspace-Root",
            "/tmp/test-workspace",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_none(), "Ping should not error");
    assert_eq!(
        mcp_response.id,
        Some(serde_json::json!(42)),
        "Should echo request ID"
    );
}

#[tokio::test]
async fn test_http_server_initialize() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send initialize request
    let request = McpRequest {
        method: "initialize".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Workspace-Root",
            "/tmp/test-workspace",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_none(), "Initialize should not error");

    let result = mcp_response.result.expect("Should have result");
    assert!(result.get("serverInfo").is_some(), "Should have serverInfo");
    assert!(
        result.get("capabilities").is_some(),
        "Should have capabilities"
    );
}

#[tokio::test]
async fn test_http_server_unknown_method() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send unknown method
    let request = McpRequest {
        method: "unknown/method".to_string(),
        params: None,
        id: Some(serde_json::json!(99)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Workspace-Root",
            "/tmp/test-workspace",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_some(), "Unknown method should error");
    let error = mcp_response.error.unwrap();
    assert_eq!(error.code, -32601, "Should be method not found error code");
}

#[tokio::test]
async fn test_http_server_with_session_header() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send request with session header
    let request = McpRequest {
        method: "ping".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Session-Id",
            "test-session-12345",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    // Request should succeed with session header
    assert!(
        mcp_response.error.is_none(),
        "Should succeed with session header"
    );
}

#[tokio::test]
async fn test_http_server_tools_call_index_status() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Call index tool with status action
    let request = McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "index",
            "arguments": {
                "action": "status",
                "collection": "test-collection"
            }
        })),
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "X-Workspace-Root",
            "/tmp/test-workspace",
        ))
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    // Tool call should return a result (even if collection doesn't exist)
    assert!(
        mcp_response.result.is_some(),
        "Should have result from tool call"
    );
}

#[tokio::test]
async fn test_http_server_tools_call_without_workspace_provenance_is_rejected() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "index",
            "arguments": {
                "action": "status",
                "collection": "test-collection"
            }
        })),
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
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

#[tokio::test]
async fn test_http_server_tools_call_missing_params_returns_invalid_params() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/call".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    let error = mcp_response
        .error
        .expect("Missing params should return error");
    assert_eq!(error.code, -32602);
}

#[tokio::test]
async fn test_http_server_tools_call_non_object_arguments_returns_invalid_params() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "index",
            "arguments": "not-an-object"
        })),
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    let error = mcp_response
        .error
        .expect("Non-object arguments should return error");
    assert_eq!(error.code, -32602);
}

#[tokio::test]
async fn test_http_server_tools_call_unknown_tool_returns_invalid_params() {
    let port = get_free_port();
    let (server_instance, _temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "does-not-exist",
            "arguments": {}
        })),
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    let error = mcp_response
        .error
        .expect("Unknown tool should return error");
    assert_eq!(error.code, -32602);
}
