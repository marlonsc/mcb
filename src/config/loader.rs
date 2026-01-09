use crate::core::error::{Error, Result};
use config::{Config as ConfigBuilder, Environment, File};
use std::path::Path;
use validator::Validate;

use super::types::Config;

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
        let builder = ConfigBuilder::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                Environment::with_prefix("MCP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = builder
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }

    pub async fn load_with_file(&self, path: &Path) -> Result<Config> {
        let builder = ConfigBuilder::builder()
            .add_source(File::from(path).required(false))
            .add_source(
                Environment::with_prefix("MCP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = builder
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }
}
