use loco_rs::prelude::Result;

use crate::constants::admin::CONFIG_ROOT;

/// Loads admin config from sea-orm-pro and returns it as JSON.
///
/// # Errors
///
/// Fails when config cannot be loaded or serialized to JSON.
pub fn load_admin_config() -> Result<serde_json::Value> {
    let cfg = sea_orm_pro::ConfigParser::new()
        .load_config(CONFIG_ROOT)
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
    serde_json::to_value(cfg).map_err(|e| loco_rs::Error::string(&e.to_string()))
}
