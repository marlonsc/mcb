use loco_rs::prelude::Result;

use mcb_infrastructure::config::{load_admin_config, resolve_admin_config_root};

/// Loads admin config from sea-orm-pro and returns it as JSON.
///
/// # Errors
///
/// Fails when config cannot be loaded or serialized to JSON.
pub fn load_admin_config_handler() -> Result<serde_json::Value> {
    let config_root = resolve_admin_config_root()
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?
        .to_string_lossy()
        .to_string();
    load_admin_config(&config_root).map_err(|e| loco_rs::Error::string(&e.to_string()))
}
