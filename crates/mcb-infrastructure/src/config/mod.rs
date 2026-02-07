//! Configuration management with hot-reload capabilities
//!
//! Provides TOML configuration loading, validation, and hot-reloading
//! for all system components. This module manages the application's
//! configuration lifecycle.

pub mod loader;
pub mod mcp_context_config;
pub mod paths;
pub mod types;
pub mod watcher;

// Re-export main configuration types
pub use types::{
    AppConfig, AuthConfig, CacheProvider, CacheSystemConfig, LoggingConfig, ServerConfig,
    ServerConfigBuilder, ServerConfigPresets, ServerCorsConfig, ServerNetworkConfig,
    ServerSslConfig, ServerTimeoutConfig, TransportMode,
};

pub use loader::ConfigLoader;
pub use mcp_context_config::{ConfigError, GitConfig, McpContextConfig};
pub use paths::{
    COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME, VCS_LOCK_FILENAME,
    VCS_REGISTRY_FILENAME, config_dir,
};
pub use watcher::{ConfigWatchEvent, ConfigWatcher, ConfigWatcherBuilder, ConfigWatcherUtils};
