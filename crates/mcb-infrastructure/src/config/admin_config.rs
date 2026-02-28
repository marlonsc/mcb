//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Admin configuration loader for sea-orm-pro admin panel.
//!
//! Provides a clean abstraction over sea-orm-pro's ConfigParser to avoid
//! direct dependencies in the controller layer.

use std::path::PathBuf;

use mcb_domain::error::{Error, Result};

/// Loads admin configuration from the specified directory.
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded or serialized.
pub fn load_admin_config(config_root: &str) -> Result<serde_json::Value> {
    let cfg = sea_orm_pro::ConfigParser::new()
        .load_config(config_root)
        .map_err(|e| Error::config(&e.to_string()))?;
    serde_json::to_value(cfg).map_err(|e| Error::config(&e.to_string()))
}

/// Resolves the admin configuration root directory.
///
/// Returns the path to the admin configuration directory, defaulting to
/// `config/pro_admin` relative to the application root.
///
/// # Errors
///
/// Returns an error if the configuration directory cannot be determined.
pub fn resolve_admin_config_root() -> Result<PathBuf> {
    // Use the standard config directory pattern
    // In production, this would be relative to the app root
    // For now, we use a simple relative path that works in development
    Ok(PathBuf::from("config/pro_admin"))
}
