use crate::config::AppConfig;
use mcb_domain::ports::EventBusProvider;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Context passed to service factory functions during DI resolution.
pub struct ServiceResolutionContext {
    /// Active database connection.
    pub db: DatabaseConnection,
    /// Shared application configuration.
    pub config: Arc<AppConfig>,
    /// Event bus for cross-service communication.
    pub event_bus: Arc<dyn EventBusProvider>,
}
