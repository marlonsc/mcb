//! Structured logging with tracing
//!
//! Provides centralized logging configuration and utilities using the tracing ecosystem.
//! This module configures structured logging with JSON output, log levels, file rotation,
//! and optional event bus forwarding for real-time SSE streaming.

mod event_bus_layer;
mod forwarder;

use std::io::IsTerminal;

use mcb_domain::error::{Error, Result};
use tracing::{Level, debug, error, info, warn};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Re-export LoggingConfig for convenience
pub use crate::config::LoggingConfig;
pub use forwarder::spawn_log_forwarder;

/// Receiver half of the log event channel.
/// The caller spawns a task to forward events to the `EventBus`.
pub type LogEventReceiver = tokio::sync::mpsc::UnboundedReceiver<mcb_domain::events::DomainEvent>;

/// Initialize logging with the provided configuration.
///
/// Returns `Some(LogEventReceiver)` when event bus forwarding is configured,
/// allowing the caller to connect it to the event bus after DI initialization.
///
/// # Errors
///
/// Returns an error if the log level is invalid or tracing subscriber initialization fails.
pub fn init_logging(config: LoggingConfig) -> Result<Option<LogEventReceiver>> {
    let level = parse_log_level(&config.level)?;
    let filter = create_log_filter(&config.level);
    let file_appender = create_file_appender(&config.file_output);

    // Create event bus layer if configured
    let (event_layer, receiver) = create_event_bus_layer(&config.event_bus_level)?;

    if config.json_format {
        init_json_logging(filter, file_appender, event_layer)?;
    } else {
        init_text_logging(filter, file_appender, event_layer)?;
    }

    info!("Logging initialized with level: {}", level);
    if receiver.is_some() {
        info!(event_bus_level = %config.event_bus_level, "Event bus log forwarding enabled");
    }
    Ok(receiver)
}

/// Create event bus layer and receiver channel pair
fn create_event_bus_layer(
    event_bus_level: &str,
) -> Result<(
    Option<event_bus_layer::EventBusLayer>,
    Option<LogEventReceiver>,
)> {
    let min_level = parse_log_level(event_bus_level)?;
    let (layer, receiver) = event_bus_layer::EventBusLayer::new(min_level);
    Ok((Some(layer), Some(receiver)))
}

/// Create log filter from configuration
///
/// Priority: `MCP_LOG` env var > config level
fn create_log_filter(level: &str) -> EnvFilter {
    EnvFilter::try_from_env("MCP_LOG").unwrap_or_else(|_| EnvFilter::new(level))
}

/// Create file appender if file output is configured
fn create_file_appender(
    file_output: &Option<std::path::PathBuf>,
) -> Option<tracing_appender::rolling::RollingFileAppender> {
    file_output.as_ref().map(|path| {
        tracing_appender::rolling::daily(
            path.parent().unwrap_or_else(|| std::path::Path::new(".")),
            path.file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("mcb")),
        )
    })
}

/// Initialize logging with JSON format
fn init_json_logging(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) -> Result<()> {
    // Terminal mode: JSON logs to stdout
    // Stdio mode: JSON logs to stderr (stdout reserved for JSON-RPC)
    if std::io::stdout().is_terminal() {
        init_json_logging_terminal(filter, file_appender, event_layer);
    } else {
        init_json_logging_stdio(filter, file_appender, event_layer);
    }
    Ok(())
}

/// Initialize JSON logging for terminal mode (output to stdout)
fn init_json_logging_terminal(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) {
    let console = fmt::layer()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    let registry = Registry::default().with(filter).with(event_layer);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .json()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(console).with(file).init();
    } else {
        registry.with(console).init();
    }
}

/// Initialize JSON logging for stdio mode (output to stderr, ANSI off)
fn init_json_logging_stdio(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) {
    let console = fmt::layer()
        .json()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    let registry = Registry::default().with(filter).with(event_layer);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .json()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(console).with(file).init();
    } else {
        registry.with(console).init();
    }
}

/// Initialize text logging for terminal mode (colored output to stdout)
fn init_text_logging_terminal(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) {
    let console = fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);
    let registry = Registry::default().with(filter).with(event_layer);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(console).with(file).init();
    } else {
        registry.with(console).init();
    }
}

/// Initialize text logging for stdio mode (plain output to stderr)
fn init_text_logging_stdio(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) {
    let console = fmt::layer()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);
    let registry = Registry::default().with(filter).with(event_layer);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(console).with(file).init();
    } else {
        registry.with(console).init();
    }
}

/// Initialize logging with text format
fn init_text_logging(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
    event_layer: Option<event_bus_layer::EventBusLayer>,
) -> Result<()> {
    // Terminal mode: colored logs to stdout
    // Stdio mode: plain logs to stderr (stdout reserved for JSON-RPC)
    if std::io::stdout().is_terminal() {
        init_text_logging_terminal(filter, file_appender, event_layer);
    } else {
        init_text_logging_stdio(filter, file_appender, event_layer);
    }
    Ok(())
}

/// Parse log level string to tracing Level
///
/// # Errors
///
/// Returns an error if the level string is not a recognized log level.
pub fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(Error::Configuration {
            message: format!("Invalid log level: {level}. Use trace, debug, info, warn, or error"),
            source: None,
        }),
    }
}

/// Log configuration loading status
pub fn log_config_loaded(config_path: &std::path::Path, success: bool) {
    if success {
        info!("Configuration loaded from {}", config_path.display());
    } else {
        warn!("Configuration file not found: {}", config_path.display());
    }
}

/// Log health check result
pub fn log_health_check(component: &str, healthy: bool, details: Option<&str>) {
    if healthy {
        debug!(component = component, "Health check passed");
    } else {
        error!(
            component = component,
            details = details.unwrap_or("Unknown failure"),
            "Health check failed"
        );
    }
}
