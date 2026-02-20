//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Configuration Management Service Logic

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::utils::json::json_to_toml;
use mcb_infrastructure::config::watcher::ConfigWatcher;

use super::SanitizedConfig;

/// Internal error type for config update operations
#[derive(Debug)]
pub enum ConfigUpdateError {
    /// Invalid configuration section
    InvalidSection,
    /// Configuration watcher is not running
    WatcherUnavailable,
    /// Configuration file path is not set
    PathUnavailable,
    /// Failed to read configuration file
    ReadFailed(String),
    /// Failed to parse configuration file
    ParseFailed(String),
    /// Invalid configuration value format
    InvalidFormat,
    /// Failed to serialize configuration
    SerializeFailed(String),
    /// Failed to write configuration file
    WriteFailed(String),
    /// Configuration reload failed after write
    ReloadFailed(String),
}

/// Validate prerequisites for config update
///
/// # Errors
/// Returns an error when section is invalid or watcher/path are unavailable.
pub fn validate_update_prerequisites(
    section: &str,
    watcher: Option<&Arc<ConfigWatcher>>,
    config_path: Option<&PathBuf>,
) -> Result<(Arc<ConfigWatcher>, PathBuf), ConfigUpdateError> {
    use super::is_valid_section;

    if !is_valid_section(section) {
        return Err(ConfigUpdateError::InvalidSection);
    }

    let watcher = watcher
        .cloned()
        .ok_or(ConfigUpdateError::WatcherUnavailable)?;
    let config_path = config_path
        .cloned()
        .ok_or(ConfigUpdateError::PathUnavailable)?;

    Ok((watcher, config_path))
}

/// Read, parse, and update the configuration
///
/// # Errors
/// Returns an error when reading, parsing, or value conversion fails.
pub fn read_update_config(
    config_path: &Path,
    section: &str,
    values: &serde_json::Value,
) -> Result<toml::Value, ConfigUpdateError> {
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| ConfigUpdateError::ReadFailed(e.to_string()))?;

    let mut config: toml::Value =
        toml::from_str(&content).map_err(|e| ConfigUpdateError::ParseFailed(e.to_string()))?;

    let toml_value = json_to_toml(values).ok_or(ConfigUpdateError::InvalidFormat)?;

    if let Some(table) = config.as_table_mut() {
        merge_section(table, section, toml_value);
    }

    Ok(config)
}

/// Merge new values into a config section
fn merge_section(
    table: &mut toml::map::Map<String, toml::Value>,
    section: &str,
    new_value: toml::Value,
) {
    let toml::Value::Table(new_table) = new_value else {
        table.insert(section.to_owned(), new_value);
        return;
    };

    let Some(existing) = table.get_mut(section).and_then(|v| v.as_table_mut()) else {
        table.insert(section.to_owned(), toml::Value::Table(new_table));
        return;
    };

    for (key, value) in new_table {
        existing.insert(key, value);
    }
}

/// Write config to file and reload
///
/// # Errors
/// Returns an error when serialization, write, or reload fails.
pub async fn write_and_reload_config(
    config_path: &Path,
    config: &toml::Value,
    watcher: &ConfigWatcher,
) -> Result<SanitizedConfig, ConfigUpdateError> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| ConfigUpdateError::SerializeFailed(e.to_string()))?;

    std::fs::write(config_path, content)
        .map_err(|e| ConfigUpdateError::WriteFailed(e.to_string()))?;

    let new_config = watcher
        .reload()
        .await
        .map_err(|e| ConfigUpdateError::ReloadFailed(e.to_string()))?;

    Ok(SanitizedConfig::from_app_config(&new_config))
}
