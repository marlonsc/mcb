//! Configuration management with hot-reload capabilities
//!
//! Provides TOML configuration loading, validation, and hot-reloading
//! for all system components. This module manages the application's
//! configuration lifecycle.

pub mod data;
pub mod loader;
pub mod mcp_context_config;
pub mod paths;
pub mod providers;
pub mod server;
pub mod types;
pub mod watcher;

pub use data::AppConfig;
pub use loader::ConfigLoader;
pub use mcp_context_config::{ConfigError, GitConfig, McpContextConfig};
pub use paths::{
    COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME, VCS_LOCK_FILENAME,
    VCS_REGISTRY_FILENAME, config_dir,
};
pub use providers::{ProviderConfigBuilder, ProviderConfigManager};
pub use server::{ServerConfigBuilder, ServerConfigPresets, ServerConfigUtils};
pub use types::{CacheProvider, LoggingConfig};
pub use watcher::{ConfigWatchEvent, ConfigWatcher, ConfigWatcherBuilder, ConfigWatcherUtils};
