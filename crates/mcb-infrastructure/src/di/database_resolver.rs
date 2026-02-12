//! Resolver for database providers.
//!
//! Handles dynamic resolution of database providers (e.g. SQLite, Postgres)
//! based on configuration strings using the `linkme` registry.

use std::path::Path;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, DatabaseProvider};
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};

use crate::config::AppConfig;

/// Resolver for database providers
pub struct DatabaseProviderResolver {
    config: Arc<AppConfig>,
}

impl DatabaseProviderResolver {
    /// Create a new resolver
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Resolve and connect to the database provider configured in AppConfig.
    ///
    /// Reads the provider name from `config.providers.database` (defaults to "sqlite").
    pub async fn resolve_and_connect(&self, path: &Path) -> Result<Arc<dyn DatabaseExecutor>> {
        let provider_name = self.config.providers.database.as_str();

        let config = DatabaseProviderConfig::new(provider_name);
        let provider = self.resolve_from_override(&config)?;

        provider.connect(path).await
    }

    /// Resolve a provider from specific configuration
    pub fn resolve_from_override(
        &self,
        config: &DatabaseProviderConfig,
    ) -> Result<Arc<dyn DatabaseProvider>> {
        resolve_database_provider(config)
            .map_err(|e| mcb_domain::error::Error::configuration(format!("Database: {e}")))
    }

    /// List all available database providers
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_domain::registry::database::list_database_providers()
    }
}
