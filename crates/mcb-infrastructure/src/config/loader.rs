//! Production-mirroring configuration loader.
//!
//! Loads `AppConfig` using the **same deserialization path as production**:
//! YAML file → parse → extract `settings` → JSON conversion →
//! `serde_json::from_value::<AppConfig>()` → `validate_app_config()`.
//!
//! This ensures tests load configuration identically to the Loco runtime
//! (see `crates/mcb/src/initializers/mcp_server.rs`).

use std::path::{Path, PathBuf};

use mcb_domain::error::{Error, Result};

use super::{AppConfig, validation::validate_app_config};

/// Load `AppConfig` using the production Loco deserialization path.
///
/// 1. Reads `LOCO_ENV` (defaults to `"test"`)
/// 2. Walks ancestors from `CARGO_MANIFEST_DIR` looking for
///    `config/{env}.local.yaml` then `config/{env}.yaml`
/// 3. Parses YAML, extracts `settings:` key
/// 4. Converts YAML → JSON via `serde_json::to_value`
/// 5. Deserializes via `serde_json::from_value::<AppConfig>()` — the production path
/// 6. Validates via `validate_app_config()`
///
/// # Errors
///
/// Returns an error if the config file is missing, unreadable, or invalid.
pub fn load_app_config() -> Result<AppConfig> {
    let env_name = std::env::var("LOCO_ENV").unwrap_or_else(|_| "test".to_owned());

    let filenames = [format!("{env_name}.local.yaml"), format!("{env_name}.yaml")];

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        for filename in &filenames {
            let candidate = dir.join("config").join(filename);
            if candidate.exists() {
                return load_from_path(&candidate);
            }
        }
    }

    Err(Error::ConfigMissing(format!(
        "No config file found for env '{env_name}'"
    )))
}

/// Load and deserialize `AppConfig` from a specific YAML file path.
fn load_from_path(path: &Path) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::config_with_source(format!("Failed to read {}", path.display()), e))?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| Error::config_with_source("Failed to parse YAML", e))?;

    let settings = yaml
        .get("settings")
        .ok_or_else(|| Error::ConfigMissing("No 'settings' key in config".into()))?;

    // Production path: YAML value → JSON value → serde_json::from_value
    // Mirrors crates/mcb/src/initializers/mcp_server.rs:44-45
    let json_settings = serde_json::to_value(settings)
        .map_err(|e| Error::config_with_source("Failed to convert YAML settings to JSON", e))?;

    let config: AppConfig = serde_json::from_value(json_settings)
        .map_err(|e| Error::config_with_source("Failed to deserialize AppConfig", e))?;

    validate_app_config(&config)?;

    Ok(config)
}
