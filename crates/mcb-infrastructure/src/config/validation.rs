use crate::config::AppConfig;
use crate::constants::auth::*;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::ProjectSettings;
use mcb_validate::find_workspace_root_from;
use std::env;

pub(crate) fn load_project_settings() -> Option<ProjectSettings> {
    let current_dir = env::current_dir().ok()?;
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

pub(crate) fn resolve_config_with_project_settings(
    mut config: AppConfig,
    settings: &ProjectSettings,
) -> AppConfig {
    if let Some(providers) = &settings.providers {
        if let Some(embedding) = &providers.embedding {
            if let Some(provider) = &embedding.provider {
                config.providers.embedding.provider = Some(provider.clone());
            }
            if let Some(model) = &embedding.model {
                config.providers.embedding.model = Some(model.clone());
            }
        }
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

pub fn validate_app_config(config: &AppConfig) -> Result<()> {
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
