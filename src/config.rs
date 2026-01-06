//! Configuration management

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub server: ServerConfig,
    pub providers: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub embedding: crate::core::types::EmbeddingConfig,
    pub vector_store: crate::core::types::VectorStoreConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            embedding: crate::core::types::EmbeddingConfig {
                provider: "mock".to_string(),
                model: "mock".to_string(),
                api_key: None,
                base_url: None,
                dimensions: Some(128),
                max_tokens: Some(512),
            },
            vector_store: crate::core::types::VectorStoreConfig {
                provider: "in-memory".to_string(),
                address: None,
                token: None,
                collection: None,
                dimensions: Some(128),
            },
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // For MVP, just return defaults
        // In future, could load from config files or environment
        Ok(Self::default())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Basic validation
        if self.name.is_empty() {
            return Err(Error::invalid_argument("Name cannot be empty"));
        }

        if self.version.is_empty() {
            return Err(Error::invalid_argument("Version cannot be empty"));
        }

        Ok(())
    }
}
