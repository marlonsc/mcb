use loco_rs::prelude::Result;

/// Default config directory when `MCB_PRO_ADMIN_CONFIG_DIR` is not set.
const DEFAULT_PRO_ADMIN_CONFIG_DIR: &str = "config/pro_admin";

/// Loads admin config from sea-orm-pro and returns it as JSON.
///
/// The config directory is read from the `MCB_PRO_ADMIN_CONFIG_DIR` environment
/// variable, falling back to `config/pro_admin` for backward compatibility.
///
/// # Errors
///
/// Fails when config cannot be loaded or serialized to JSON.
pub fn load_admin_config() -> Result<serde_json::Value> {
    let config_root = std::env::var("MCB_PRO_ADMIN_CONFIG_DIR")
        .unwrap_or_else(|_| DEFAULT_PRO_ADMIN_CONFIG_DIR.to_owned());
    let cfg = sea_orm_pro::ConfigParser::new()
        .load_config(&config_root)
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
    serde_json::to_value(cfg).map_err(|e| loco_rs::Error::string(&e.to_string()))
}
