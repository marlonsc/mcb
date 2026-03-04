//! Configuration helpers — simplified access via CA/DI.
//!
//! These helpers resolve `ConfigProvider` from the linkme registry
//! and delegate to it. Consumers never import infrastructure types
//! directly — everything flows through `Box<dyn Any + Send + Sync>`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use mcb_domain::utils::config;
//!
//! // Load from YAML files (tests, CLI)
//! let config_any = config::load("loco_yaml")?;
//!
//! // Deserialize from serde_json::Value (composition root / Loco)
//! let config_any = config::from_value("loco_yaml", &settings)?;
//!
//! // Validate an already-loaded config
//! config::validate("loco_yaml", &*config_any)?;
//!
//! // Load + downcast in one call (composition root convenience)
//! let app_config: MyConfigType = config::load_as("loco_yaml")?;
//! ```

use std::any::Any;

use crate::error::{Error, Result};
use crate::registry::config::{ConfigProviderConfig, resolve_config_provider};

/// Default provider name for the project.
pub const DEFAULT_PROVIDER: &str = mcb_utils::constants::DEFAULT_CONFIG_PROVIDER;

/// Load application configuration via CA/DI registry.
///
/// Resolves the `ConfigProvider` by name and calls `load_config()`.
/// Returns the configuration as `Box<dyn Any + Send + Sync>`.
///
/// # Errors
///
/// Returns an error if the provider cannot be resolved or config loading fails.
pub fn load(provider_name: &str) -> Result<Box<dyn Any + Send + Sync>> {
    let provider = resolve_config_provider(&ConfigProviderConfig::new(provider_name))?;
    provider.load_config()
}

/// Load application configuration using the default provider.
///
/// Shorthand for `load(DEFAULT_PROVIDER)`.
///
/// # Errors
///
/// Returns an error if the provider cannot be resolved or config loading fails.
pub fn load_default() -> Result<Box<dyn Any + Send + Sync>> {
    load(DEFAULT_PROVIDER)
}

/// Deserialize configuration from a pre-loaded `serde_json::Value`.
///
/// This is the production path: Loco provides `AppContext.config.settings`
/// as `serde_json::Value`, which is passed here for deserialization + validation.
///
/// # Errors
///
/// Returns an error if deserialization or validation fails.
pub fn from_value(provider_name: &str, settings: &dyn Any) -> Result<Box<dyn Any + Send + Sync>> {
    let provider = resolve_config_provider(&ConfigProviderConfig::new(provider_name))?;
    provider.deserialize_from_value(settings)
}

/// Validate an already-loaded configuration.
///
/// # Errors
///
/// Returns an error if any configuration constraint is violated.
pub fn validate(provider_name: &str, config: &dyn Any) -> Result<()> {
    let provider = resolve_config_provider(&ConfigProviderConfig::new(provider_name))?;
    provider.validate_config(config)
}

/// Load and downcast configuration to a specific type in one call.
///
/// Convenience for composition roots and tests that know the concrete type.
///
/// # Errors
///
/// Returns an error if loading fails or the concrete type doesn't match.
pub fn load_as<T: 'static + Send + Sync>(provider_name: &str) -> Result<T> {
    let config_any = load(provider_name)?;
    config_any
        .downcast::<T>()
        .map(|b| *b)
        .map_err(|_| Error::internal("ConfigProvider returned unexpected type (downcast failed)"))
}

/// Load and downcast using the default provider.
///
/// Shorthand for `load_as::<T>(DEFAULT_PROVIDER)`.
///
/// # Errors
///
/// Returns an error if loading fails or the concrete type doesn't match.
pub fn load_default_as<T: 'static + Send + Sync>() -> Result<T> {
    load_as::<T>(DEFAULT_PROVIDER)
}

/// Deserialize from value and downcast in one call.
///
/// Production composition root convenience.
///
/// # Errors
///
/// Returns an error if deserialization, validation, or downcast fails.
pub fn from_value_as<T: 'static + Send + Sync>(
    provider_name: &str,
    settings: &dyn Any,
) -> Result<T> {
    let config_any = from_value(provider_name, settings)?;
    config_any
        .downcast::<T>()
        .map(|b| *b)
        .map_err(|_| Error::internal("ConfigProvider returned unexpected type (downcast failed)"))
}
