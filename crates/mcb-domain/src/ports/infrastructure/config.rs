//! Configuration provider ports.

/// Domain port for configuration providers.
///
/// Implementations load application configuration from a source
/// (YAML files, environment variables, etc.) and return it as
/// a type-erased `Box<dyn Any + Send + Sync>`.
///
/// The concrete config type (e.g. `AppConfig`) lives in the infrastructure
/// layer; the domain remains free of serde/YAML dependencies.
///
/// # Usage patterns
///
/// - **Loco initializer** (composition root): call [`deserialize_from_value`](Self::deserialize_from_value)
///   with the `serde_json::Value` from `AppContext.config.settings`.
/// - **Tests**: call [`load_config`](Self::load_config) to load from YAML files.
/// - **Validation**: call [`validate_config`](Self::validate_config) on a config
///   previously deserialized or loaded.
pub trait ConfigProvider: Send + Sync {
    /// Load application configuration from the default source (YAML files).
    ///
    /// Walks ancestor directories from `CARGO_MANIFEST_DIR` looking for
    /// `config/{LOCO_ENV}.yaml`. Deserializes AND validates before returning.
    ///
    /// # Errors
    ///
    /// Returns an error if the config source is missing, unreadable,
    /// or fails validation.
    fn load_config(&self) -> crate::error::Result<Box<dyn std::any::Any + Send + Sync>>;

    /// Deserialize configuration from a pre-loaded `serde_json::Value`.
    ///
    /// This is the production path used by Loco initializers: the Loco
    /// framework provides `AppContext.config.settings` as `serde_json::Value`,
    /// which is passed here for deserialization and validation.
    ///
    /// # Arguments
    ///
    /// * `settings` - An opaque `&dyn Any` that **must** contain a
    ///   `serde_json::Value`. The implementation downcasts internally.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a `serde_json::Value`,
    /// if deserialization fails, or if validation fails.
    fn deserialize_from_value(
        &self,
        settings: &dyn std::any::Any,
    ) -> crate::error::Result<Box<dyn std::any::Any + Send + Sync>>;

    /// Validate an already-loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration constraint is violated.
    fn validate_config(&self, config: &dyn std::any::Any) -> crate::error::Result<()>;

    /// Human-readable name of this configuration provider.
    fn provider_name(&self) -> &str;
}
