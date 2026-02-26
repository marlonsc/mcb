use crate::config::AppConfig;
use mcb_domain::ports::EventBusProvider;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ServiceResolutionContext {
    pub db: DatabaseConnection,
    pub config: Arc<AppConfig>,
    pub event_bus: Arc<dyn EventBusProvider>,
}
