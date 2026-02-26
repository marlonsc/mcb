//! Configuration Management - Type-safe, layered, Validated
//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Provides YAML configuration loading (Loco convention), validation, and
//! type-safe configuration for all system components.

mod mcp_context_config;
pub mod paths;
pub mod test_builder;
pub mod types;
pub mod validation;

// Re-export main configuration types
pub use types::{
    AppConfig, AuthConfig, CacheProvider, CacheSystemConfig, DatabaseConfig,
    DatabaseConfigContainer, LoggingConfig, ServerConfig, ServerCorsConfig, ServerNetworkConfig,
    ServerSslConfig, ServerTimeoutConfig, TransportMode,
};

pub use mcp_context_config::{GitConfig, McpContextConfig};
pub use paths::{
    COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME, VCS_LOCK_FILENAME,
    VCS_REGISTRY_FILENAME, config_dir,
};
pub use test_builder::TestConfigBuilder;
pub use validation::validate_app_config;
