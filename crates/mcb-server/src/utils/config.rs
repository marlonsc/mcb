use mcb_infrastructure::config::{AppConfig, ConfigLoader};

/// Load server startup configuration or panic with startup context.
#[must_use]
pub fn load_startup_config_or_panic() -> AppConfig {
    match ConfigLoader::new().load() {
        Ok(config) => config,
        Err(error) => panic!("startup: configuration file must be loadable: {error}"),
    }
}
