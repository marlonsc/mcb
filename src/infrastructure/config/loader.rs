use crate::domain::error::{Error, Result};
use config::{Config as ConfigBuilder, Environment, FileFormat};
use std::path::Path;
use validator::Validate;

use super::types::Config;

/// Embedded default configuration from config/default.toml
/// This is the single source of truth for default values in the binary.
/// Works from any working directory because it's compiled into the binary.
const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config/default.toml");

#[derive(Debug, Clone, Copy)]
pub struct ConfigLoader;

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self
    }

    pub async fn load(&self) -> Result<Config> {
        // Start with embedded default config (source of truth for defaults)
        let mut builder = ConfigBuilder::builder().add_source(config::File::from_str(
            DEFAULT_CONFIG_TOML,
            FileFormat::Toml,
        ));

        // Layer 2: User configuration from XDG standard location (if exists)
        let config_dir = dirs::config_dir();
        if let Some(dir) = config_dir {
            let user_config_path = dir.join("mcp-context-browser").join("config.toml");
            if user_config_path.exists() {
                builder = builder.add_source(config::File::from(user_config_path).required(false));
            }
        }

        // Layer 3: Environment variables (highest priority)
        builder = builder.add_source(
            Environment::with_prefix("MCP")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = config
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }

    pub async fn load_with_file(&self, path: &Path) -> Result<Config> {
        // Start with embedded default config
        let mut builder = ConfigBuilder::builder()
            .add_source(config::File::from_str(
                DEFAULT_CONFIG_TOML,
                FileFormat::Toml,
            ))
            // Override with specified file
            .add_source(config::File::from(path).required(false));

        // Environment variables still have highest priority
        builder = builder.add_source(
            Environment::with_prefix("MCP")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = config
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }
}
