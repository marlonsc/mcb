//! Configuration provider implementation.

use std::any::Any;
use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ConfigProvider;

use super::app;
use super::loader;
use super::validation;

/// Loco YAML configuration provider.
///
/// Registered via `#[linkme::distributed_slice]` in `CONFIG_PROVIDERS`.
pub(crate) struct LocoYamlConfigProvider;

impl ConfigProvider for LocoYamlConfigProvider {
    fn load_config(&self) -> Result<Box<dyn Any + Send + Sync>> {
        let config = loader::load_app_config()?;
        Ok(Box::new(config))
    }

    fn deserialize_from_value(&self, settings: &dyn Any) -> Result<Box<dyn Any + Send + Sync>> {
        let json_value = settings
            .downcast_ref::<serde_json::Value>()
            .ok_or_else(|| {
                Error::internal(
                    "ConfigProvider::deserialize_from_value: expected serde_json::Value",
                )
            })?;

        let config: app::AppConfig = serde_json::from_value(json_value.clone()).map_err(|e| {
            Error::config_with_source("Failed to deserialize AppConfig from JSON", e)
        })?;

        validation::validate_app_config(&config)?;

        Ok(Box::new(config))
    }

    fn validate_config(&self, config: &dyn Any) -> Result<()> {
        let app_config = config.downcast_ref::<app::AppConfig>().ok_or_else(|| {
            Error::internal("ConfigProvider::validate_config: expected AppConfig")
        })?;
        validation::validate_app_config(app_config)
    }

    fn provider_name(&self) -> &str {
        mcb_utils::constants::DEFAULT_CONFIG_PROVIDER
    }
}

mcb_domain::register_config_provider!(
    mcb_utils::constants::DEFAULT_CONFIG_PROVIDER,
    "YAML configuration loader following Loco conventions",
    |_config| Ok(Arc::new(LocoYamlConfigProvider))
);
