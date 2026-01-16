//! DI Component Registry Tests
//!
//! Tests for the component registry and service locator pattern.

use mcb_infrastructure::cache::CacheProviderFactory;
use mcb_infrastructure::crypto::CryptoService;
use mcb_infrastructure::di::registry::{ComponentRegistry, ServiceLocator};
use mcb_infrastructure::health::HealthRegistry;

#[derive(Clone, Debug, PartialEq)]
struct TestComponent {
    value: String,
}

#[tokio::test]
async fn test_component_registry() {
    let registry = ComponentRegistry::new();

    let component = TestComponent {
        value: "test".to_string(),
    };

    // Register component
    registry.register(component.clone()).await.unwrap();

    // Get component
    let retrieved: TestComponent = registry.get().await.unwrap();
    assert_eq!(retrieved, component);

    // Check existence
    assert!(registry.has::<TestComponent>().await);

    // Count components
    assert_eq!(registry.count().await, 1);

    // Remove component
    registry.remove::<TestComponent>().await.unwrap();
    assert!(!registry.has::<TestComponent>().await);
}

#[tokio::test]
async fn test_component_registry_duplicate_registration() {
    let registry = ComponentRegistry::new();

    let component = TestComponent {
        value: "test".to_string(),
    };

    // First registration should succeed
    registry.register(component.clone()).await.unwrap();

    // Second registration should fail
    let result = registry.register(component).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_component_registry_clear() {
    let registry = ComponentRegistry::new();

    registry
        .register(TestComponent {
            value: "test".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(registry.count().await, 1);

    registry.clear().await;
    assert_eq!(registry.count().await, 0);
}

#[tokio::test]
async fn test_service_locator() {
    let locator = ServiceLocator::new();

    let cache = CacheProviderFactory::create_null();
    let crypto = CryptoService::new(CryptoService::generate_master_key()).unwrap();
    let health = HealthRegistry::new();

    locator
        .register_infrastructure_components(cache.clone(), crypto.clone(), health.clone())
        .await
        .unwrap();

    // Test that components can be retrieved
    let retrieved_cache = locator.cache().await.unwrap();
    let _retrieved_crypto = locator.crypto().await.unwrap();
    let _retrieved_health = locator.health().await.unwrap();

    // Components should be accessible
    assert!(retrieved_cache
        .get::<_, String>("test")
        .await
        .unwrap()
        .is_none());
}
