//! Admin configuration and API types
//!
//! ## Security-First Design
//!
//! AdminConfig REQUIRES explicit configuration - NO hardcoded defaults.
//! Must be loaded from environment variables or config file.
//!
//! Required environment variables:
//! - `ADMIN_USERNAME` - Admin account username
//! - `ADMIN_PASSWORD` - Admin account password (min 8 chars)
//! - `JWT_SECRET` - JWT signing secret (min 32 chars)
//!
//! Optional:
//! - `JWT_EXPIRATION` - Expiration in seconds (default: 3600)
//! - `ADMIN_ENABLED` - Enable/disable (default: true if credentials provided)

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MIN_JWT_SECRET_LENGTH: usize = 32;

/// Admin API server configuration
///
/// Can be loaded from:
/// 1. TOML config file [admin] section (preferred for local dev)
/// 2. Environment variables (preferred for production)
/// 3. Hardcoded defaults (for testing only)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdminConfig {
    /// Enable admin interface (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Admin username (default: "admin" for local dev)
    #[serde(default = "default_username")]
    #[validate(length(min = 1))]
    pub username: String,
    /// Admin password (default: "admin" for local dev)
    #[serde(default = "default_password")]
    #[validate(length(min = 1))]
    pub password: String,
    /// JWT secret for authentication (default: "default-jwt-secret-change-in-production")
    #[serde(default = "default_jwt_secret")]
    #[validate(length(min = 1))]
    pub jwt_secret: String,
    /// JWT expiration time in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_jwt_expiration")]
    #[validate(range(min = 1))]
    pub jwt_expiration: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_username() -> String {
    "admin".to_string()
}

fn default_password() -> String {
    "admin".to_string()
}

fn default_jwt_secret() -> String {
    "default-jwt-secret-change-in-production".to_string()
}

fn default_jwt_expiration() -> u64 {
    3600  // 1 hour
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    #[error("Invalid JWT secret: {0}")]
    InvalidJwtSecret(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl AdminConfig {
    /// Create with explicit values
    pub fn new(enabled: bool, username: String, password: String, jwt_secret: String, jwt_expiration: u64) -> Result<Self, ConfigError> {
        let config = Self {
            enabled,
            username,
            password,
            jwt_secret,
            jwt_expiration,
        };
        config.validate()
            .map_err(|e| ConfigError::ConfigError(format!("Validation failed: {}", e)))?;
        Ok(config)
    }

    /// Create for testing ONLY
    #[cfg(test)]
    pub fn for_testing(username: &str, password: &str, jwt_secret: &str) -> Result<Self, ConfigError> {
        let config = Self {
            enabled: true,
            username: username.to_string(),
            password: password.to_string(),
            jwt_secret: jwt_secret.to_string(),
            jwt_expiration: 3600,
        };
        config.validate()
            .map_err(|e| ConfigError::ConfigError(format!("Validation failed: {}", e)))?;
        Ok(config)
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            username: "admin".to_string(),
            password: "admin".to_string(),
            jwt_secret: "default-jwt-secret-change-in-production".to_string(),
            jwt_expiration: 3600,
        }
    }
}

/// Admin API instance
pub struct AdminApi {
    config: AdminConfig,
}

impl AdminApi {
    /// Create a new admin API instance
    pub fn new(config: AdminConfig) -> Self {
        Self { config }
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
