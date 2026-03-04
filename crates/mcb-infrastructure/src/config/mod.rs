//! Configuration — concrete implementation (CA/DI).
//!
//! All access goes through `mcb_domain::utils::config` helpers →
//! `mcb_domain::registry::config::resolve_config_provider()`.
//!
//! Types are `pub` (needed for downcast at composition root).
//! Loader/validation are private implementation details.

pub mod app;
pub mod infrastructure;
mod loader;
pub mod mode;
pub mod system;
mod validation;

// ---------------------------------------------------------------------------
// ConfigProvider Concrete Implementation
// ---------------------------------------------------------------------------

use std::any::Any;
use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ConfigProvider;

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

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::config::{CONFIG_PROVIDERS, ConfigProviderEntry};

#[allow(unsafe_code)]
#[linkme::distributed_slice(CONFIG_PROVIDERS)]
static LOCO_YAML_CONFIG_ENTRY: ConfigProviderEntry = ConfigProviderEntry {
    name: mcb_utils::constants::DEFAULT_CONFIG_PROVIDER,
    description: "YAML configuration loader following Loco conventions",
    build: |_config| Ok(Arc::new(LocoYamlConfigProvider)),
};
