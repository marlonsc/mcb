use std::path::PathBuf;

use clap::Args;
use loco_rs::boot::{self, ServeParams, StartMode};
use loco_rs::controller::middleware;
use loco_rs::environment::Environment;
use mcb_server::McbApp;

use loco_rs::app::Hooks;
use loco_rs::config;

/// Arguments for the `serve` subcommand.
#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    /// Path to the configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Run as server daemon (HTTP only, no stdio).
    #[arg(long, help = "Run as server daemon (HTTP only, no stdio)")]
    pub server: bool,
}

impl ServeArgs {
    /// # Errors
    /// Returns an error if config loading, DB setup, or Loco boot fails.
    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server {
            // SAFETY: called once at startup before any other threads are spawned.
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var("MCB_NO_STDIO", "1");
            }
        }

        let mcb_config = load_mcb_config(self.config.as_deref())?;
        let db_path = resolve_db_path(&mcb_config)?;
        let http_port = i32::from(mcb_config.server.network.port);
        let http_binding = mcb_config.server.network.host.clone();
        let db_uri = sqlite_uri(&db_path);

        let loco_config = build_loco_config(http_port, http_binding.clone(), db_uri);
        let environment = Environment::Development;

        let boot_result =
            McbApp::boot(StartMode::ServerOnly, &environment, loco_config.clone()).await?;
        let serve = ServeParams {
            port: loco_config.server.port,
            binding: loco_config.server.binding.clone(),
        };

        boot::start::<McbApp>(boot_result, serve, false).await?;
        Ok(())
    }
}

fn load_mcb_config(
    config_path: Option<&std::path::Path>,
) -> Result<mcb_infrastructure::config::AppConfig, Box<dyn std::error::Error>> {
    let loader = match config_path {
        Some(path) => mcb_infrastructure::config::ConfigLoader::new().with_config_path(path),
        None => mcb_infrastructure::config::ConfigLoader::new(),
    };
    Ok(loader.load()?)
}

fn resolve_db_path(
    config: &mcb_infrastructure::config::AppConfig,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let db_config = config
        .providers
        .database
        .configs
        .get(mcb_infrastructure::constants::providers::DEFAULT_DB_CONFIG_NAME)
        .ok_or("providers.database.configs.default is required")?;
    let path = db_config
        .path
        .clone()
        .ok_or("providers.database.configs.default.path is required")?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(path)
}

fn sqlite_uri(db_path: &std::path::Path) -> String {
    format!("sqlite://{}?mode=rwc", db_path.display())
}

fn build_loco_config(http_port: i32, binding: String, db_uri: String) -> config::Config {
    config::Config {
        logger: config::Logger {
            enable: true,
            pretty_backtrace: true,
            level: loco_rs::logger::LogLevel::Info,
            format: loco_rs::logger::Format::Compact,
            override_filter: None,
            file_appender: None,
        },
        server: config::Server {
            binding,
            port: http_port,
            host: format!("http://127.0.0.1:{http_port}"),
            ident: None,
            middlewares: middleware::Config::default(),
        },
        database: config::Database {
            uri: db_uri,
            enable_logging: false,
            min_connections: 1,
            max_connections: 5,
            connect_timeout: 500,
            idle_timeout: 500,
            acquire_timeout: None,
            auto_migrate: true,
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
