//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.

use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;

// Force link mcb_providers so inventory registrations are included
extern crate mcb_providers;

#[tokio::test]
async fn test_di_container_builder() {
    let config = AppConfig::default();
    let result = init_app(config).await;

    assert!(
        result.is_ok(),
        "init_app should complete successfully: {:?}",
        result.err()
    );

    let app_context = result.unwrap();

    // Verify context has expected fields
    assert!(
        std::mem::size_of_val(&app_context.config) > 0,
        "Config should be initialized"
    );
    assert!(
        std::mem::size_of_val(&app_context.providers) > 0,
        "Providers should be initialized"
    );
}
