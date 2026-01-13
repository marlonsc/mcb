//! Infrastructure DI Module Implementation
//!
//! Contains system metrics, service providers, and core infrastructure.

use shaku::module;

use super::traits::InfrastructureModule;
use crate::infrastructure::di::factory::ServiceProvider;
use crate::infrastructure::metrics::system::SystemMetricsCollector;

module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [SystemMetricsCollector, ServiceProvider],
        providers = []
    }
}
