use mcb_infrastructure::config::{AppConfig, ConfigLoader};

/// Load server startup configuration.
///
/// # Errors
/// Returns an error when configuration loading fails.
pub fn load_startup_config() -> Result<AppConfig, mcb_domain::error::Error> {
    ConfigLoader::new().load()
}

/// Load startup configuration and terminate the process when loading fails.
#[must_use]
pub fn load_startup_config_or_exit() -> AppConfig {
    match load_startup_config() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(error = %error, "startup config unavailable, aborting startup");
            std::process::exit(2);
        }
    }
}
