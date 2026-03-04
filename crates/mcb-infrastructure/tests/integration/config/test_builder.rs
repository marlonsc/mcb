//! Fluent builder for test configurations (CA/DI pattern).
//!
//! Resolves `ConfigProvider` via `mcb_domain::registry::config` and uses it
//! to load + validate configuration. No direct access to infrastructure
//! functions — all config operations go through the registry.
//!
//! # Usage
//!
//! ```rust,ignore
//! let (config, _temp) = TestConfigBuilder::new()?
//!     .with_temp_db("my-test.db")?
//!     .with_fastembed_shared_cache()?
//!     .build()?;
//! ```
//!
//! **All test config customization MUST go through this builder.**
//! Never mutate `AppConfig` fields directly in test code.

use std::path::PathBuf;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ConfigProvider;
use mcb_domain::registry::config::{ConfigProviderConfig, resolve_config_provider};
use mcb_infrastructure::config::app::{AppConfig, DatabaseConfig};
use tempfile::TempDir;

/// Resolve the default `ConfigProvider` via CA/DI registry.
fn resolve_default_config_provider() -> Result<std::sync::Arc<dyn ConfigProvider>> {
    resolve_config_provider(&ConfigProviderConfig::new(
        mcb_domain::utils::config::DEFAULT_PROVIDER,
    ))
}

/// Fluent builder for test configurations.
///
/// Uses `ConfigProvider` (resolved via CA/DI registry) to load
/// config from YAML and validate after applying overrides.
pub struct TestConfigBuilder {
    config: AppConfig,
    config_provider: std::sync::Arc<dyn ConfigProvider>,
    temp_dir: Option<TempDir>,
}

impl TestConfigBuilder {
    /// Create a builder seeded from Loco YAML config via CA/DI.
    ///
    /// Resolves `ConfigProvider` from the registry, then calls
    /// `load_config()` to load + validate from YAML files.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be resolved or config is invalid.
    pub fn new() -> Result<Self> {
        let config_provider = resolve_default_config_provider()?;
        let config_any = config_provider.load_config()?;
        let config = config_any
            .downcast::<AppConfig>()
            .map_err(|_| Error::internal("ConfigProvider returned unexpected type"))?;
        Ok(Self {
            config: *config,
            config_provider,
            temp_dir: None,
        })
    }

    /// Override the default database path with a fresh temporary directory.
    ///
    /// Creates a `TempDir` and sets `providers.database.configs["default"]`
    /// to a `SQLite` database inside it. The `TempDir` is returned from
    /// [`build`](Self::build) so the caller can keep it alive.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the temporary directory fails.
    pub fn with_temp_db(mut self, db_name: &str) -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join(db_name);
        self.config.providers.database.configs.insert(
            "default".to_owned(),
            DatabaseConfig {
                provider: "sqlite".to_owned(),
                path: Some(db_path),
            },
        );
        self.temp_dir = Some(temp_dir);
        Ok(self)
    }

    /// Override the default database path with an explicit path.
    #[must_use]
    pub fn with_db_path(mut self, path: PathBuf) -> Self {
        self.config.providers.database.configs.insert(
            "default".to_owned(),
            DatabaseConfig {
                provider: "sqlite".to_owned(),
                path: Some(path),
            },
        );
        self
    }

    /// Override the embedding provider cache directory.
    #[must_use]
    pub fn with_embedding_cache(mut self, path: PathBuf) -> Self {
        self.config.providers.embedding.cache_dir = Some(path);
        self
    }

    /// Use the shared `FastEmbed` ONNX model cache directory.
    ///
    /// Reads `MCB_FASTEMBED_TEST_CACHE_DIR` env var or falls back to
    /// `$TMPDIR/mcb-fastembed-test-cache`. Ensures the directory exists.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the cache directory fails.
    pub fn with_fastembed_shared_cache(mut self) -> Result<Self> {
        let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR").map_or_else(
            || std::env::temp_dir().join("mcb-fastembed-test-cache"),
            PathBuf::from,
        );
        std::fs::create_dir_all(&cache_dir)?;
        self.config.providers.embedding.cache_dir = Some(cache_dir);
        Ok(self)
    }

    /// Direct access to the config for advanced overrides.
    ///
    /// Prefer the typed `with_*` methods when possible.
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Validate and finalize the configuration.
    ///
    /// Uses the resolved `ConfigProvider` to re-validate after overrides.
    /// Returns `(AppConfig, Option<TempDir>)`. The caller MUST keep the
    /// `TempDir` alive for the duration of the test.
    ///
    /// # Errors
    ///
    /// Returns an error if the overridden config fails validation (fail-fast).
    pub fn build(self) -> Result<(AppConfig, Option<TempDir>)> {
        // Validate via CA/DI — same path as production
        self.config_provider.validate_config(&self.config)?;
        Ok((self.config, self.temp_dir))
    }
}
