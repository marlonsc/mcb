//! DI Module Tests
//!
//! Tests for Shaku-based dependency injection modules.

use mcb_infrastructure::config::ConfigBuilder;
use mcb_infrastructure::di::modules::InfrastructureModule;

#[tokio::test]
async fn test_infrastructure_module_creation() {
    let config = ConfigBuilder::new().build();
    let _module = InfrastructureModule::new(config);
    // Module creation should succeed - if we get here, it worked
}
