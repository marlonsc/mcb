//! Admin Service DI Module Implementation
//!
//! Contains admin service with dependencies on infrastructure and server modules.

use shaku::module;

use super::traits::{AdminModule, InfrastructureModule, ServerModule};
use crate::admin::service::AdminServiceImpl;
use crate::adapters::http_client::HttpClientProvider;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::metrics::PerformanceMetricsInterface;
use crate::server::operations::IndexingOperationsInterface;

module! {
    pub AdminModuleImpl: AdminModule {
        components = [AdminServiceImpl],
        providers = [],

        use InfrastructureModule {
            components = [SystemMetricsCollectorInterface, ServiceProviderInterface],
            providers = []
        },

        use ServerModule {
            components = [PerformanceMetricsInterface, IndexingOperationsInterface],
            providers = []
        },

        use super::traits::AdaptersModule {
            components = [HttpClientProvider],
            providers = []
        }
    }
}
