//! Server Initialization
//!
//! Handles server startup, dependency injection setup, and graceful shutdown.
//! Integrates with the infrastructure layer for configuration and DI container setup.
//!
//! # Architecture (Clean Architecture + Handle-based DI)
//!
//! The server initialization follows a handle-based DI approach:
//!
//! 1. **Provider Handles** (Infrastructure): `RwLock` wrappers for runtime-swappable providers
//! 2. **Runtime Factory** (Application): Creates domain services with providers from handles
//!
//! Production providers are resolved via linkme registry using `AppConfig`,
//! wrapped in handles, and can be switched at runtime via admin API.
//!
//! # Operating Modes
//!
//! MCB supports three operating modes:
//!
//! | Mode | Trigger | Description |
//! | ------ | --------- | ------------- |
//! | **Server** | `--server` flag | HTTP daemon accepting client connections |
//! | **Standalone** | Config `mode.type = "standalone"` | Local providers, stdio transport |
//! | **Client** | Config `mode.type = "client"` | Connects to remote server via HTTP |
//!
//! # Configuration
//!
//! Mode selection via config file (`~/.config/mcb/mcb.toml`):
//! ```toml
//! [mode]
//! type = "client"                         # "standalone" or "client"
//! server_url = "http://127.0.0.1:8080"   # For client mode
//! ```

use std::path::Path;
use std::sync::Arc;

use mcb_infrastructure::config::{
    AppConfig,
    types::{OperatingMode, TransportMode},
};
use tracing::{error, info, warn};

use crate::McpServer;
use crate::admin::auth::AdminAuthConfig;
use crate::admin::browse_handlers::BrowseState;
use crate::admin::handlers::AdminState;
use crate::transport::http::{HttpTransport, HttpTransportConfig};
use crate::transport::stdio::StdioServerExt;

// =============================================================================
// Main Entry Point
// =============================================================================

/// Main entry point - dispatches to the appropriate operating mode
///
/// # Arguments
///
/// * `config_path` - Optional path to configuration file
/// * `server_mode` - If true, runs as server daemon (ignores config mode)
///
/// # Operating Mode Selection
///
/// 1. If `server_mode` is true → Run as server daemon (HTTP + optional stdio)
/// 2. Otherwise, check `config.mode.mode_type`:
///    - `Standalone` → Run with local providers, stdio transport
///    - `Client` → Connect to remote server via HTTP
pub async fn run(
    config_path: Option<&Path>,
    server_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
    let log_receiver = mcb_infrastructure::logging::init_logging(config.logging.clone())?;

    if server_mode {
        // Explicit server mode via --server flag
        run_server_mode(
            config,
            config_path.map(std::path::Path::to_path_buf),
            log_receiver,
        )
        .await
    } else {
        // Check config for operating mode
        match config.mode.mode_type {
            OperatingMode::Standalone => run_standalone(config, log_receiver).await,
            OperatingMode::Client => run_client(config).await,
        }
    }
}

// =============================================================================
// Operating Modes
// =============================================================================

/// Run as server daemon (HTTP + optional stdio based on `transport_mode`)
async fn run_server_mode(
    config: AppConfig,
    config_path: Option<std::path::PathBuf>,
    log_receiver: Option<mcb_infrastructure::logging::LogEventReceiver>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        transport_mode = ?config.server.transport_mode,
        host = %config.server.network.host,
        port = %config.server.network.port,
        "Starting MCB server daemon (single port)"
    );

    let transport_mode = config.server.transport_mode;
    let http_host = config.server.network.host.clone();
    let http_port = config.server.network.port;

    let (server, app_context) = create_mcp_server(config.clone(), "server").await?;
    info!("MCP server initialized successfully");

    let event_bus = app_context.event_bus();

    // Connect log event channel to the event bus for SSE streaming
    if let Some(receiver) = log_receiver {
        mcb_infrastructure::logging::spawn_log_forwarder(receiver, event_bus.clone());
        info!("Log event forwarder connected to event bus");
    }

    // Create admin state for consolidated single-port operation
    // Initialize ConfigWatcher for hot-reload support
    let config_watcher = if let Some(ref path) = config_path {
        mcb_infrastructure::config::watcher::ConfigWatcher::new(
            path.clone(),
            config.clone(),
            event_bus.clone(),
        )
        .await
        .ok()
        .map(std::sync::Arc::new)
    } else {
        None
    };

    // Initialize ServiceManager for lifecycle operations
    let service_manager =
        mcb_infrastructure::infrastructure::ServiceManager::new(app_context.event_bus());

    // Register services from AppContext (Arc clone is O(1) — atomic refcount increment)
    for service in &app_context.lifecycle_services {
        service_manager.register(service.clone());
    }

    let admin_state = AdminState {
        metrics: app_context.performance(),
        indexing: app_context.indexing(),
        config_watcher,
        current_config: config.clone(),
        config_path,
        shutdown_coordinator: Some(app_context.shutdown()),
        shutdown_timeout_secs: 30,
        event_bus,
        service_manager: Some(std::sync::Arc::new(service_manager)),
        cache: Some(app_context.cache_handle().get()),
        project_workflow: Some(server.project_workflow_repository()),
        vcs_entity: Some(server.vcs_entity_repository()),
        plan_entity: Some(server.plan_entity_repository()),
        issue_entity: Some(server.issue_entity_repository()),
        org_entity: Some(server.org_entity_repository()),
        tool_handlers: Some(server.tool_handlers()),
    };

    let browse_state = BrowseState {
        browser: app_context.vector_store_handle().get(),
        highlight_service: app_context.highlight_service(),
    };

    let auth_config = std::sync::Arc::new(AdminAuthConfig::default());

    match transport_mode {
        TransportMode::Stdio => {
            warn!(
                "Server mode with stdio-only transport. Consider using 'hybrid' for client connections."
            );
            run_stdio_transport(server).await
        }
        TransportMode::Http => {
            info!(host = %http_host, port = http_port, "Starting HTTP transport (MCP + Admin)");
            run_http_transport_with_admin(
                server,
                &http_host,
                http_port,
                admin_state,
                auth_config,
                Some(browse_state),
            )
            .await
        }
        TransportMode::Hybrid => {
            info!(
                host = %http_host,
                port = http_port,
                "Starting hybrid transport (stdio + HTTP with Admin)"
            );
            run_hybrid_transport_with_admin(
                server,
                &http_host,
                http_port,
                admin_state,
                auth_config,
                Some(browse_state),
            )
            .await
        }
    }
}

/// Run in standalone mode with local providers
///
/// This is the default mode when no `--server` flag is provided and
/// `config.mode.type = "standalone"`. MCB runs with local providers
/// and communicates via stdio (for Claude Code integration).
async fn run_standalone(
    config: AppConfig,
    log_receiver: Option<mcb_infrastructure::logging::LogEventReceiver>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        transport_mode = ?config.server.transport_mode,
        "Starting MCB standalone mode"
    );

    let transport_mode = config.server.transport_mode;
    let http_host = config.server.network.host.clone();
    let http_port = config.server.network.port;

    let (server, app_context) = create_mcp_server(config, "standalone").await?;
    info!("MCP server initialized successfully");

    // Connect log event channel to the event bus for SSE streaming
    if let Some(receiver) = log_receiver {
        mcb_infrastructure::logging::spawn_log_forwarder(receiver, app_context.event_bus());
    }

    start_transport(server, transport_mode, &http_host, http_port).await
}

/// Run in client mode, connecting to a remote MCB server
///
/// This mode is activated when `config.mode.type = "client"`. MCB acts as
/// a stdio-to-HTTP bridge: it reads MCP requests from stdin, forwards them
/// to the remote server via HTTP, and writes responses to stdout.
async fn run_client(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let server_url = &config.mode.server_url;
    let session_prefix = config.mode.session_prefix.as_deref();

    info!(
        server_url = %server_url,
        session_prefix = ?session_prefix,
        timeout_secs = config.mode.timeout_secs,
        "Starting MCB client mode"
    );

    use crate::transport::http_client::HttpClientTransport;

    let cfg_session_id = config
        .mode
        .session_id
        .clone()
        .filter(|v| !v.trim().is_empty());
    let cfg_session_file = config
        .mode
        .session_file
        .clone()
        .filter(|v| !v.trim().is_empty());

    let client = HttpClientTransport::new_with_session_source(
        server_url.clone(),
        session_prefix.map(String::from),
        std::time::Duration::from_secs(config.mode.timeout_secs),
        cfg_session_id,
        cfg_session_file,
    )?;

    client.run().await
}

// =============================================================================
// Configuration Loading
// =============================================================================

/// Load configuration from optional path
fn load_config(config_path: Option<&Path>) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let loader = match config_path {
        Some(path) => mcb_infrastructure::config::ConfigLoader::new().with_config_path(path),
        None => mcb_infrastructure::config::ConfigLoader::new(),
    };
    Ok(loader.load()?)
}

// =============================================================================
// Server Creation
// =============================================================================

/// Create and configure the MCP server with all services
async fn create_mcp_server(
    config: AppConfig,
    execution_flow: &str,
) -> Result<(McpServer, mcb_infrastructure::di::bootstrap::AppContext), Box<dyn std::error::Error>>
{
    let app_context = mcb_infrastructure::di::bootstrap::init_app(config.clone()).await?;
    let services = app_context.build_domain_services().await?;

    let mcp_services = crate::mcp_server::McpServices {
        indexing: services.indexing_service,
        context: services.context_service,
        search: services.search_service,
        validation: services.validation_service,
        memory: services.memory_service,
        agent_session: services.agent_session_service,
        project: services.project_service,
        project_workflow: services.project_repository,
        vcs: services.vcs_provider,
        vcs_entity: services.vcs_entity_repository,
        plan_entity: services.plan_entity_repository,
        issue_entity: services.issue_entity_repository,
        org_entity: services.org_entity_repository,
    };
    let server = McpServer::from_services(mcp_services, Some(execution_flow.to_owned()));

    Ok((server, app_context))
}

// =============================================================================
// Transport Management
// =============================================================================

/// Start the appropriate transport based on configuration
async fn start_transport(
    server: McpServer,
    transport_mode: TransportMode,
    http_host: &str,
    http_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match transport_mode {
        TransportMode::Stdio => {
            info!("Starting stdio transport");
            run_stdio_transport(server).await
        }
        TransportMode::Http => {
            info!(host = %http_host, port = http_port, "Starting HTTP transport");
            run_http_transport(server, http_host, http_port).await
        }
        TransportMode::Hybrid => {
            info!(
                host = %http_host,
                port = http_port,
                "Starting hybrid transport (stdio + HTTP)"
            );
            run_hybrid_transport(server, http_host, http_port).await
        }
    }
}

/// Run the server with stdio transport only
///
/// This is the traditional MCP transport mode, communicating over stdin/stdout.
/// Used for CLI tools and IDE integrations like Claude Code.
async fn run_stdio_transport(server: McpServer) -> Result<(), Box<dyn std::error::Error>> {
    server.serve_stdio().await
}

/// Run HTTP transport with MCP endpoints only (no admin)
async fn run_http_transport(
    server: McpServer,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_config = HttpTransportConfig {
        host: host.to_owned(),
        port,
        enable_cors: true,
    };

    let http_transport = HttpTransport::new(http_config, Arc::new(server));
    http_transport
        .start()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e })
}

/// Run HTTP transport with consolidated MCP + Admin endpoints
async fn run_http_transport_with_admin(
    server: McpServer,
    host: &str,
    port: u16,
    admin_state: AdminState,
    auth_config: std::sync::Arc<AdminAuthConfig>,
    browse_state: Option<BrowseState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_config = HttpTransportConfig {
        host: host.to_owned(),
        port,
        enable_cors: true,
    };

    let http_transport = HttpTransport::new(http_config, Arc::new(server)).with_admin(
        admin_state,
        auth_config,
        browse_state,
    );
    http_transport
        .start()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e })
}

/// Run hybrid transport (stdio + HTTP) without admin
async fn run_hybrid_transport(
    server: McpServer,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdio_server = server.clone();
    let http_server = Arc::new(server);
    let http_host = host.to_owned();

    let stdio_handle = tokio::spawn(async move {
        info!("Hybrid: starting stdio transport");
        if let Err(e) = stdio_server.serve_stdio().await {
            error!(error = %e, "Hybrid: stdio transport failed");
        }
        info!("Hybrid: stdio transport finished");
    });

    let http_handle = tokio::spawn(async move {
        info!("Hybrid: starting HTTP transport on {}:{}", http_host, port);
        let http_config = HttpTransportConfig {
            host: http_host,
            port,
            enable_cors: true,
        };

        let http_transport = HttpTransport::new(http_config, http_server);
        if let Err(e) = http_transport.start().await {
            error!(error = %e, "Hybrid: HTTP transport failed");
        }
        info!("Hybrid: HTTP transport finished");
    });

    let (stdio_result, http_result) = tokio::join!(stdio_handle, http_handle);

    if let Err(e) = stdio_result {
        error!(error = %e, "Hybrid: stdio transport task panicked");
    }
    if let Err(e) = http_result {
        error!(error = %e, "Hybrid: HTTP transport task panicked");
    }

    Ok(())
}

/// Run hybrid transport (stdio + HTTP) with consolidated Admin endpoints
async fn run_hybrid_transport_with_admin(
    server: McpServer,
    host: &str,
    port: u16,
    admin_state: AdminState,
    auth_config: std::sync::Arc<AdminAuthConfig>,
    browse_state: Option<BrowseState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdio_server = server.clone();
    let http_server = Arc::new(server);
    let http_host = host.to_owned();

    let stdio_handle = tokio::spawn(async move {
        info!("Hybrid: starting stdio transport");
        if let Err(e) = stdio_server.serve_stdio().await {
            error!(error = %e, "Hybrid: stdio transport failed");
        }
        info!("Hybrid: stdio transport finished");
    });

    let http_handle = tokio::spawn(async move {
        info!(
            "Hybrid: starting HTTP+Admin transport on {}:{}",
            http_host, port
        );
        let http_config = HttpTransportConfig {
            host: http_host,
            port,
            enable_cors: true,
        };

        let http_transport = HttpTransport::new(http_config, http_server).with_admin(
            admin_state,
            auth_config,
            browse_state,
        );
        if let Err(e) = http_transport.start().await {
            error!(error = %e, "Hybrid: HTTP+Admin transport failed");
        }
        info!("Hybrid: HTTP+Admin transport finished");
    });

    let (stdio_result, http_result) = tokio::join!(stdio_handle, http_handle);

    if let Err(e) = stdio_result {
        error!(error = %e, "Hybrid: stdio transport task panicked");
    }
    if let Err(e) = http_result {
        error!(error = %e, "Hybrid: HTTP transport task panicked");
    }

    Ok(())
}
