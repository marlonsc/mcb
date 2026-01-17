//! Infrastructure DI Module Implementation
//!
//! Contains system metrics, service providers, event bus, auth, and core infrastructure.

#![allow(missing_docs)]

use shaku::module;

use super::traits::InfrastructureModule;
use crate::infrastructure::auth::AuthService;
use crate::infrastructure::di::factory::ServiceProvider;
use crate::infrastructure::events::EventBus;
use crate::infrastructure::metrics::system::SystemMetricsCollector;
use crate::infrastructure::snapshot::SnapshotManager;
use crate::infrastructure::sync::SyncManager;

module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            SystemMetricsCollector,
            ServiceProvider,
            EventBus,
            AuthService,
            SnapshotManager,
            SyncManager
        ],
        providers = []
    }
}
