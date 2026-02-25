//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Fluent builder for test configurations.
//!
//! Loads from `config/{env}.yaml` (Loco convention), then applies
//! test-specific mutations. If any override produces an invalid config,
//! `build()` fails — the test cannot start with bad config.
//!
//! # Usage
//!
//! ```rust,ignore
//! let (config, _temp) = TestConfigBuilder::new()?
//!     .with_temp_db("my-test.db")?
//!     .with_fastembed_shared_cache()?
//!     .build()?;
//!
//! let ctx = init_app(config).await?;
//! ```
//!
//! **All test config customization MUST go through this builder.**
//! Never mutate `AppConfig` fields directly in test code.

use std::path::PathBuf;

use mcb_domain::error::Result;
use tempfile::TempDir;

use super::{AppConfig, ConfigLoader, DatabaseConfig};

/// Fluent builder for test configurations.
///
/// Loads from Loco YAML config on construction, then applies typed overrides.
/// Call [`build`](Self::build) to validate and finalize.
pub struct TestConfigBuilder {
    config: AppConfig,
    temp_dir: Option<TempDir>,
}

impl TestConfigBuilder {
    /// Create a builder seeded from Loco YAML config.
    ///
    /// # Errors
    ///
    /// Returns an error if the default config file is missing or invalid.
    pub fn new() -> Result<Self> {
        let config = ConfigLoader::new().load()?;
        Ok(Self {
            config,
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
    /// Returns `(AppConfig, Option<TempDir>)`. The caller MUST keep the
    /// `TempDir` alive for the duration of the test — dropping it deletes
    /// the temporary database.
    ///
    /// # Errors
    ///
    /// Returns an error if the overridden config fails validation (fail-fast).
    pub fn build(self) -> Result<(AppConfig, Option<TempDir>)> {
        // Re-validate after overrides to ensure fail-fast on bad config
        ConfigLoader::validate_for_test(&self.config)?;
        Ok((self.config, self.temp_dir))
    }
}
