use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use loco_rs::app::{AppContext, Hooks};
use loco_rs::bgworker::Queue;
use loco_rs::boot::{self, BootResult, ServeParams, StartMode, create_app};
use loco_rs::config;
use loco_rs::controller::{AppRoutes, Routes};
use loco_rs::environment::Environment;
use loco_rs::prelude::{Response, format, get, post};
use loco_rs::task::Tasks;
use loco_rs::{Result, controller::middleware};
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::ServiceExt;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::transport::stdio;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use serde::Serialize;
use serde_json::json;

const DEFAULT_HTTP_PORT: i32 = 3017;
const DEFAULT_HTTP_BINDING: &str = "127.0.0.1";
const DEFAULT_DB_PATH: &str = "./spike.sqlite";

#[derive(Debug)]
struct App;

#[derive(Debug)]
struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

#[derive(Serialize)]
struct Health {
    ok: bool,
}

#[derive(Serialize)]
struct NoteWriteResult {
    inserted: bool,
    note_count: i64,
}

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        "loco-rmcp-spike"
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: config::Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes().add_route(
            Routes::new()
                .add("/health", get(health))
                .add("/notes", post(write_note)),
        )
    }

    async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        let db = ctx.db.clone();
        tokio::spawn(async move {
            if let Err(error) = run_mcp_stdio(db).await {
                tracing::error!(error = %error, "mcp stdio task stopped");
            }
        });
        Ok(router)
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    async fn seed(_ctx: &AppContext, _base: &std::path::Path) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct SpikeMcpServer {
    db: DatabaseConnection,
}

impl SpikeMcpServer {
    fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ServerHandler for SpikeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "loco-rmcp-spike".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Spike MCP server: one index tool that reads note count from shared SQLite DB"
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let schema = json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        });

        let input_schema = schema
            .as_object()
            .cloned()
            .ok_or_else(|| McpError::internal_error("invalid schema", None))?;

        Ok(ListToolsResult {
            tools: vec![Tool {
                name: Cow::Borrowed("index"),
                title: Some("Index".to_string()),
                description: Some(Cow::Borrowed("Read note count from shared SQLite database")),
                input_schema: Arc::new(input_schema),
                output_schema: None,
                annotations: None,
                icons: None,
                execution: None,
                meta: Default::default(),
            }],
            meta: Default::default(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if request.name.as_ref() != "index" {
            return Err(McpError::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            ));
        }

        let count = count_notes(&self.db)
            .await
            .map_err(|_| McpError::internal_error("failed to query note count", None))?;

        let payload = json!({
            "tool": "index",
            "note_count": count
        });
        Ok(CallToolResult::success(vec![Content::text(
            payload.to_string(),
        )]))
    }
}

async fn run_mcp_stdio(
    db: DatabaseConnection,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = SpikeMcpServer::new(db);
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| format!("failed to start MCP stdio: {e:?}"))?;
    service
        .waiting()
        .await
        .map_err(|e| format!("MCP stdio service failure: {e:?}"))?;
    Ok(())
}

async fn ensure_schema(db: &DatabaseConnection) -> Result<()> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL)".to_string(),
    ))
    .await?;
    Ok(())
}

async fn count_notes(db: &DatabaseConnection) -> Result<i64> {
    ensure_schema(db).await?;
    let row = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(*) AS note_count FROM notes".to_string(),
        ))
        .await?;

    let count = row
        .as_ref()
        .and_then(|value| value.try_get::<i64>("", "note_count").ok())
        .unwrap_or(0);
    Ok(count)
}

async fn health() -> Result<Response> {
    format::json(Health { ok: true })
}

async fn write_note(State(ctx): State<AppContext>) -> Result<Response> {
    ensure_schema(&ctx.db).await?;
    ctx.db
        .execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO notes (title) VALUES ('http-write')".to_string(),
        ))
        .await?;

    let count = count_notes(&ctx.db).await?;
    format::json(NoteWriteResult {
        inserted: true,
        note_count: count,
    })
}

fn build_config(http_port: i32, db_uri: String) -> config::Config {
    config::Config {
        logger: config::Logger {
            enable: false,
            pretty_backtrace: true,
            level: loco_rs::logger::LogLevel::Off,
            format: loco_rs::logger::Format::Compact,
            override_filter: None,
            file_appender: None,
        },
        server: config::Server {
            binding: DEFAULT_HTTP_BINDING.to_string(),
            port: http_port,
            host: format!("http://{DEFAULT_HTTP_BINDING}:{http_port}"),
            ident: None,
            middlewares: middleware::Config::default(),
        },
        database: config::Database {
            uri: db_uri,
            enable_logging: false,
            min_connections: 1,
            max_connections: 4,
            connect_timeout: 500,
            idle_timeout: 500,
            acquire_timeout: None,
            auto_migrate: false,
            dangerously_truncate: false,
            dangerously_recreate: false,
            run_on_start: None,
        },
        queue: None,
        auth: None,
        workers: config::Workers {
            mode: config::WorkerMode::ForegroundBlocking,
        },
        mailer: None,
        initializers: None,
        settings: None,
        scheduler: None,
        cache: config::CacheConfig::InMem(config::InMemCacheConfig {
            max_capacity: 32 * 1024 * 1024,
        }),
    }
}

fn sqlite_uri_from_path(db_path: &str) -> String {
    if db_path.starts_with("sqlite:") {
        db_path.to_string()
    } else {
        format!("sqlite://{db_path}?mode=rwc")
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let http_port = std::env::var("SPIKE_HTTP_PORT")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(DEFAULT_HTTP_PORT);
    let db_path = std::env::var("SPIKE_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());
    let db_uri = sqlite_uri_from_path(&db_path);
    let config = build_config(http_port, db_uri);

    let environment = Environment::Development;
    let boot = App::boot(StartMode::ServerOnly, &environment, config.clone()).await?;
    let serve = ServeParams {
        port: config.server.port,
        binding: config.server.binding.clone(),
    };

    boot::start::<App>(boot, serve, true).await?;
    Ok(())
}
