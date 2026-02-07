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
pub use loader::{ConfigBuilder, ConfigLoader};
pub use mcp_context_config::{ConfigError, GitConfig, McpContextConfig};
pub use paths::{
    COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME, VCS_LOCK_FILENAME,
    VCS_REGISTRY_FILENAME, collection_mapping_lock_path, collection_mapping_path, config_dir,
    vcs_registry_lock_path, vcs_registry_path,
};
pub use providers::{ProviderConfigBuilder, ProviderConfigManager};
pub use server::{ServerConfigBuilder, ServerConfigPresets, ServerConfigUtils};
pub use watcher::{ConfigWatchEvent, ConfigWatcher, ConfigWatcherBuilder, ConfigWatcherUtils};
