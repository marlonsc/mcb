use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(length(min = 1))]
    pub host: String,
    #[validate(range(min = 1))]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        panic!("[FATAL] ServerConfig::default() should never be called - all config must come from config/default.toml embedded in binary")
    }
}
