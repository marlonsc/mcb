//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_infrastructure::config::{AppConfig, ConfigLoader};

/// Load server startup configuration.
///
/// # Errors
/// Returns an error when configuration loading fails.
pub fn load_startup_config() -> Result<AppConfig, mcb_domain::error::Error> {
    ConfigLoader::new().load()
}

/// Load startup configuration, falling back to defaults when loading fails.
#[must_use]
pub fn load_startup_config_or_default() -> AppConfig {
    match load_startup_config() {
        Ok(config) => config,
        Err(error) => {
            tracing::warn!(error = %error, "startup config unavailable, using defaults");
            AppConfig::fallback()
        }
    }
}
