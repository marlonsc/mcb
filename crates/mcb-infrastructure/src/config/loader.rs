//! Configuration loader
//!
//! Handles loading configuration from various sources including
//! TOML files, environment variables, and default values.

use std::env;
use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use mcb_domain::error::{Error, Result};

use crate::config::AppConfig;
use crate::constants::auth::*;
use crate::constants::config::*;
use crate::error_ext::ErrorContext;
use crate::logging::log_config_loaded;

/// Configuration loader service
#[derive(Clone)]
pub struct ConfigLoader {
    /// Configuration file path
    config_path: Option<PathBuf>,

    /// Environment prefix
    env_prefix: String,
}

impl ConfigLoader {
    /// Create a new configuration loader with default settings
    pub fn new() -> Self {
        Self {
            config_path: None,
            env_prefix: CONFIG_ENV_PREFIX.to_string(),
        }
    }

    /// Set the configuration file path
    pub fn with_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Load configuration from all sources
    ///
    /// Configuration sources are merged in this order (later sources override earlier):
    /// 1. Default TOML configuration file (`config/default.toml`) (required)
    /// 2. Optional TOML override file (`--config`) (if provided)
    /// 3. Environment variables with `MCP__` prefix (e.g., `MCP__SERVER__NETWORK__PORT`)
    pub fn load(&self) -> Result<AppConfig> {
        let default_path = Self::find_defaults_file_path().ok_or_else(|| {
            Error::ConfigMissing(
                "Default configuration file not found. Expected config/default.toml".to_string(),
            )
        })?;
        log_config_loaded(&default_path, true);

        // Source of truth starts from canonical defaults file only.
        // Runtime must not rely on hardcoded struct defaults.
        let mut figment = Figment::new().merge(Toml::file(&default_path));

        if let Some(config_path) = &self.config_path {
            if !config_path.exists() {
                log_config_loaded(config_path, false);
                return Err(Error::ConfigMissing(format!(
                    "Configuration file not found: {}",
                    config_path.display()
                )));
            }

            if config_path != &default_path {
                figment = figment.merge(Toml::file(config_path));
                log_config_loaded(config_path, true);
            }
        }

        // Add environment variables
        // Uses double underscore as separator for nested keys (e.g., MCP__SERVER__PORT)
        // Prefix is MCP__ (double underscore) to match mcp-config.json env format
        // lowercase(true) converts PROVIDERS__EMBEDDING to providers.embedding
        figment = figment.merge(
            Env::prefixed(&format!("{}{}", self.env_prefix, CONFIG_ENV_SEPARATOR))
                .split(CONFIG_ENV_SEPARATOR)
                .lowercase(true),
        );

        // Extract and deserialize configuration
        let app_config: AppConfig = figment
            .extract()
            .context("Failed to extract configuration")?;

        // Validate configuration
        self.validate_config(&app_config)?;

        Ok(app_config)
    }

    /// Reload configuration (useful for hot-reloading)
    pub fn reload(&self) -> Result<AppConfig> {
        self.load()
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, config: &AppConfig, path: P) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

        std::fs::write(path.as_ref(), toml_string).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the current configuration file path
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Find canonical defaults file path to try
    ///
    /// Per configuration policy, runtime defaults must come from a defaults TOML file,
    /// not from hardcoded Rust constants.
    fn find_defaults_file_path() -> Option<PathBuf> {
        let current_dir = env::current_dir().ok()?;

        // Search current directory and its ancestors for config/default.toml
        for dir in current_dir.ancestors() {
            let candidate = dir.join("config").join("default.toml");
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // Search from crate location up to workspace root for config/default.toml
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for dir in manifest_dir.ancestors() {
            let candidate = dir.join("config").join("default.toml");
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    /// Validate configuration values
    fn validate_config(&self, config: &AppConfig) -> Result<()> {
        validate_app_config(config)
    }
}

/// Validate application configuration
///
/// Performs comprehensive validation of all configuration sections.
fn validate_app_config(config: &AppConfig) -> Result<()> {
    validate_server_config(config)?;
    validate_auth_config(config)?;
    validate_cache_config(config)?;
    validate_limits_config(config)?;
    validate_daemon_config(config)?;
    validate_backup_config(config)?;
    validate_operations_config(config)?;
    Ok(())
}

fn validate_server_config(config: &AppConfig) -> Result<()> {
    if config.server.network.port == 0 {
        return Err(Error::ConfigInvalid {
            key: "server.network.port".to_string(),
            message: "Server port cannot be 0".to_string(),
        });
    }
    if config.server.ssl.https
        && (config.server.ssl.ssl_cert_path.is_none() || config.server.ssl.ssl_key_path.is_none())
    {
        return Err(Error::ConfigInvalid {
            key: "server.ssl".to_string(),
            message: "SSL certificate and key paths are required when HTTPS is enabled".to_string(),
        });
    }
    Ok(())
}

fn validate_auth_config(config: &AppConfig) -> Result<()> {
    if config.auth.enabled {
        if config.auth.jwt.secret.is_empty() {
            return Err(Error::ConfigInvalid {
                key: "auth.jwt.secret".to_string(),
                message: "JWT secret cannot be empty when authentication is enabled".to_string(),
            });
        }
        if config.auth.jwt.secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(Error::Configuration {
                message: format!(
                    "JWT secret should be at least {} characters long",
                    MIN_JWT_SECRET_LENGTH
                ),
                source: None,
            });
        }
    }
    Ok(())
}

fn validate_cache_config(config: &AppConfig) -> Result<()> {
    if config.system.infrastructure.cache.enabled
        && config.system.infrastructure.cache.default_ttl_secs == 0
    {
        return Err(Error::Configuration {
            message: "Cache TTL cannot be 0 when cache is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_limits_config(config: &AppConfig) -> Result<()> {
    if config.system.infrastructure.limits.memory_limit == 0 {
        return Err(Error::Configuration {
            message: "Memory limit cannot be 0".to_string(),
            source: None,
        });
    }
    if config.system.infrastructure.limits.cpu_limit == 0 {
        return Err(Error::Configuration {
            message: "CPU limit cannot be 0".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_daemon_config(config: &AppConfig) -> Result<()> {
    if config.operations_daemon.daemon.enabled
        && config.operations_daemon.daemon.max_restart_attempts == 0
    {
        return Err(Error::Configuration {
            message: "Maximum restart attempts cannot be 0 when daemon is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_backup_config(config: &AppConfig) -> Result<()> {
    if config.system.data.backup.enabled && config.system.data.backup.interval_secs == 0 {
        return Err(Error::Configuration {
            message: "Backup interval cannot be 0 when backup is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_operations_config(config: &AppConfig) -> Result<()> {
    if config.operations_daemon.operations.tracking_enabled {
        if config.operations_daemon.operations.cleanup_interval_secs == 0 {
            return Err(Error::Configuration {
                message: "Operations cleanup interval cannot be 0 when tracking is enabled"
                    .to_string(),
                source: None,
            });
        }
        if config.operations_daemon.operations.retention_secs == 0 {
            return Err(Error::Configuration {
                message: "Operations retention period cannot be 0 when tracking is enabled"
                    .to_string(),
                source: None,
            });
        }
    }
    Ok(())
}

/// Returns default ConfigLoader for loading application configuration from files
impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}
