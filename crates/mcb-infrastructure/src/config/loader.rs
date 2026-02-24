//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Configuration loader — YAML-based (Loco convention)
//!
//! Loads `AppConfig` from Loco YAML configuration files. Application settings
//! live under the `settings:` key in `config/{env}.yaml`.
//!
//! Environment is resolved from `LOCO_ENV` / `RAILS_ENV` / `NODE_ENV` (default: `development`).

use std::env;
use std::path::{Path, PathBuf};

use mcb_domain::error::{Error, Result};

use crate::config::AppConfig;
use crate::config::TransportMode;
use crate::constants::auth::*;
use crate::error_ext::ErrorContext;

use mcb_domain::value_objects::ProjectSettings;
use mcb_validate::find_workspace_root_from;

/// Configuration loader service
///
/// Reads Loco YAML config files and extracts the `settings:` section
/// as `AppConfig`. Follows Loco's file and environment conventions.
#[derive(Clone)]
pub struct ConfigLoader {
    /// Optional explicit config file path (overrides environment resolution)
    config_path: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader with default settings
    #[must_use]
    pub fn new() -> Self {
        Self { config_path: None }
    }

    /// Set an explicit configuration file path (overrides env-based resolution)
    #[must_use]
    pub fn with_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Load configuration from YAML
    ///
    /// Resolution order:
    /// 1. Explicit path (via `with_config_path`)
    /// 2. `config/{env}.local.yaml` (highest priority override)
    /// 3. `config/{env}.yaml` (standard config)
    ///
    /// Environment is resolved from `LOCO_ENV` / `RAILS_ENV` / `NODE_ENV`, defaulting to `development`.
    ///
    /// # Errors
    ///
    /// Returns an error if no config file is found, parsing fails, or validation detects invalid values.
    pub fn load(&self) -> Result<AppConfig> {
        let yaml_path = self.find_yaml_config_path()?;
        mcb_domain::info!(
            "config",
            "Configuration loaded",
            &yaml_path.display().to_string()
        );

        let content =
            std::fs::read_to_string(&yaml_path).context("Failed to read YAML config file")?;
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&content).context("Failed to parse YAML config")?;

        let settings = yaml.get("settings").ok_or_else(|| {
            Error::ConfigMissing("No 'settings' key found in YAML configuration file".to_owned())
        })?;

        let app_config: AppConfig = serde_yaml::from_value(settings.clone())
            .context("Failed to deserialize settings into AppConfig")?;

        // Validate configuration
        Self::validate_config(&app_config)?;

        // Apply project settings if available
        let mut app_config = app_config;
        if let Some(project_settings) = Self::load_project_settings() {
            app_config = resolve_config_with_project_settings(app_config, &project_settings);
            app_config.project_settings = Some(project_settings);
        }

        Ok(app_config)
    }

    /// Reload configuration (re-reads from disk)
    ///
    /// # Errors
    ///
    /// Returns an error if configuration loading fails.
    pub fn reload(&self) -> Result<AppConfig> {
        self.load()
    }

    /// Save configuration to a YAML file (wrapped under `settings:` key)
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file writing fails.
    pub fn save_to_file<P: AsRef<Path>>(&self, config: &AppConfig, path: P) -> Result<()> {
        let settings_value = serde_yaml::to_value(config).context("Failed to serialize config")?;
        let mut root = serde_yaml::Mapping::new();
        root.insert(
            serde_yaml::Value::String("settings".to_owned()),
            settings_value,
        );
        let yaml_string =
            serde_yaml::to_string(&root).context("Failed to serialize config to YAML")?;
        std::fs::write(path.as_ref(), yaml_string).context("Failed to write config file")?;
        Ok(())
    }

    /// Get the current configuration file path
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Find the YAML config file following Loco conventions.
    ///
    /// Resolution:
    /// 1. Explicit `config_path` (if set)
    /// 2. Search `config/{env}.local.yaml` then `config/{env}.yaml`
    ///    from current directory upward
    /// 3. Search from `CARGO_MANIFEST_DIR` upward (workspace root)
    fn find_yaml_config_path(&self) -> Result<PathBuf> {
        // 1. Explicit path takes precedence
        if let Some(path) = &self.config_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(Error::ConfigMissing(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        // Determine environment (same logic as Loco)
        let env_name = env::var("LOCO_ENV")
            .or_else(|_| env::var("RAILS_ENV"))
            .or_else(|_| env::var("NODE_ENV"))
            .unwrap_or_else(|_| "development".to_owned());

        let filenames = [format!("{env_name}.local.yaml"), format!("{env_name}.yaml")];

        // 2. Search from current directory upward
        if let Ok(current_dir) = env::current_dir() {
            for dir in current_dir.ancestors() {
                for filename in &filenames {
                    let candidate = dir.join("config").join(filename);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        // 3. Search from CARGO_MANIFEST_DIR upward (for tests run from crate dirs)
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for dir in manifest_dir.ancestors() {
            for filename in &filenames {
                let candidate = dir.join("config").join(filename);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        Err(Error::ConfigMissing(format!(
            "No YAML configuration file found for environment '{env_name}'. \
             Expected config/{env_name}.yaml"
        )))
    }

    /// Validate configuration values
    fn validate_config(config: &AppConfig) -> Result<()> {
        validate_app_config(config)
    }

    /// Validate an `AppConfig` that was built or mutated outside the loader.
    ///
    /// Used by [`TestConfigBuilder`](super::test_builder::TestConfigBuilder)
    /// to re-validate after applying test overrides (fail-fast).
    ///
    /// # Errors
    ///
    /// Returns an error if any config value is invalid.
    pub fn validate_for_test(config: &AppConfig) -> Result<()> {
        validate_app_config(config)
    }

    /// Load project-specific settings from workspace root
    fn load_project_settings() -> Option<ProjectSettings> {
        let current_dir = env::current_dir().ok()?;
        // Try to find workspace root using mcb-validate logic (cargo workspace or git root)
        let root = find_workspace_root_from(&current_dir).unwrap_or(current_dir);

        let possible_paths = vec![
            root.join("mcb.yaml"),
            root.join(".mcb/config.yaml"),
            root.join("mcb.yml"),
            root.join(".mcb/config.yml"),
        ];

        for path in possible_paths {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_yaml::from_str(&content) {
                        Ok(settings) => {
                            mcb_domain::info!(
                                "config",
                                "Configuration loaded",
                                &path.display().to_string()
                            );
                            return Some(settings);
                        }
                        Err(e) => {
                            mcb_domain::warn!(
                                "config_loader",
                                "Failed to parse project settings",
                                &format!("path = {}, error = {}", path.display(), e)
                            );
                        }
                    },
                    Err(e) => {
                        mcb_domain::warn!(
                            "config_loader",
                            "Failed to read project settings",
                            &format!("path = {}, error = {}", path.display(), e)
                        );
                    }
                }
            }
        }
        None
    }
}

/// Helper to merge project settings into `AppConfig`.
fn resolve_config_with_project_settings(
    mut config: AppConfig,
    settings: &ProjectSettings,
) -> AppConfig {
    if let Some(providers) = &settings.providers {
        // Override Embedding Provider
        if let Some(embedding) = &providers.embedding {
            if let Some(provider) = &embedding.provider {
                config.providers.embedding.provider = Some(provider.clone());
            }
            if let Some(model) = &embedding.model {
                config.providers.embedding.model = Some(model.clone());
            }
        }
        // Override Vector Store Provider
        if let Some(vector_store) = &providers.vector_store {
            if let Some(provider) = &vector_store.provider {
                config.providers.vector_store.provider = Some(provider.clone());
            }
            if let Some(collection) = &vector_store.collection {
                config.providers.vector_store.collection = Some(collection.clone());
            }
        }
    }
    config
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
    // Port 0 is valid: OS assigns an ephemeral port (used in tests and stdio-only mode).
    if matches!(config.server.transport_mode, TransportMode::Http) {
        return Err(Error::ConfigInvalid {
            key: "server.transport_mode".to_owned(),
            message: "transport_mode=http is not supported. Use stdio or hybrid (stdio bridge to local daemon).".to_owned(),
        });
    }
    if config.server.ssl.https
        && (config.server.ssl.ssl_cert_path.is_none() || config.server.ssl.ssl_key_path.is_none())
    {
        return Err(Error::ConfigInvalid {
            key: "server.ssl".to_owned(),
            message: "SSL certificate and key paths are required when HTTPS is enabled".to_owned(),
        });
    }
    Ok(())
}

fn validate_auth_config(config: &AppConfig) -> Result<()> {
    if config.auth.enabled {
        if config.auth.jwt.secret.is_empty() {
            return Err(Error::ConfigInvalid {
                key: "auth.jwt.secret".to_owned(),
                message: "JWT secret cannot be empty when authentication is enabled".to_owned(),
            });
        }
        if config.auth.jwt.secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(Error::Configuration {
                message: format!(
                    "JWT secret should be at least {MIN_JWT_SECRET_LENGTH} characters long"
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
            message: "Cache TTL cannot be 0 when cache is enabled".to_owned(),
            source: None,
        });
    }
    Ok(())
}

fn validate_limits_config(config: &AppConfig) -> Result<()> {
    if config.system.infrastructure.limits.memory_limit == 0 {
        return Err(Error::Configuration {
            message: "Memory limit cannot be 0".to_owned(),
            source: None,
        });
    }
    if config.system.infrastructure.limits.cpu_limit == 0 {
        return Err(Error::Configuration {
            message: "CPU limit cannot be 0".to_owned(),
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
            message: "Maximum restart attempts cannot be 0 when daemon is enabled".to_owned(),
            source: None,
        });
    }
    Ok(())
}

fn validate_backup_config(config: &AppConfig) -> Result<()> {
    if config.system.data.backup.enabled && config.system.data.backup.interval_secs == 0 {
        return Err(Error::Configuration {
            message: "Backup interval cannot be 0 when backup is enabled".to_owned(),
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
                    .to_owned(),
                source: None,
            });
        }
        if config.operations_daemon.operations.retention_secs == 0 {
            return Err(Error::Configuration {
                message: "Operations retention period cannot be 0 when tracking is enabled"
                    .to_owned(),
                source: None,
            });
        }
    }
    Ok(())
}

/// Returns default `ConfigLoader` for loading application configuration from files
impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}
