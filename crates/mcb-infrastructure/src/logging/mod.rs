//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Structured logging with tracing
//!
//! Provides centralized logging configuration and utilities using the tracing ecosystem.
//! This module configures structured logging with JSON output, log levels, file rotation,
//! and optional event bus forwarding for real-time SSE streaming.

mod event_bus_layer;
mod forwarder;
mod sensitive;
mod setup;

// Re-export LoggingConfig for convenience
pub use crate::config::LoggingConfig;
pub use forwarder::spawn_log_forwarder;
pub use sensitive::{log_facade_shim, log_operation, set_global_operation_logger};
pub use setup::{
    LogEventReceiver, init_logging, log_config_loaded, log_health_check, parse_log_level,
};
