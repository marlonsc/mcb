//! Common macros for infrastructure layer

// ============================================================================
// Provider Resolver Implementation
// ============================================================================

/// Implement `ProviderResolver<P, C>` for a concrete resolver type
///
/// Delegates all trait methods to the resolver's inherent methods.
macro_rules! impl_provider_resolver {
    ($resolver:ty, $provider:ty, $config:ty) => {
        impl ProviderResolver<$provider, $config> for $resolver {
            fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<$provider>> {
                <$resolver>::resolve_from_config(self)
            }

            fn resolve_from_override(
                &self,
                config: &$config,
            ) -> mcb_domain::error::Result<Arc<$provider>> {
                <$resolver>::resolve_from_override(self, config)
            }

            fn list_available(&self) -> Vec<(&'static str, &'static str)> {
                <$resolver>::list_available(self)
            }
        }
    };
}

// ============================================================================
// Admin Interface Implementation
// ============================================================================

/// Implement an admin interface trait for a concrete admin service type
///
/// Two variants:
/// - Basic: `list_providers`, `switch_provider`, `reload_from_config`
/// - `with_current_provider`: adds `current_provider()` via handle
macro_rules! impl_admin_interface {
    ($service:ty, $trait:ty, $config:ty) => {
        impl $trait for $service {
            fn list_providers(&self) -> Vec<ProviderInfo> {
                AdminService::list_providers(self)
            }

            fn switch_provider(&self, config: $config) -> Result<(), String> {
                AdminService::switch_provider(self, &config)
            }

            fn reload_from_config(&self) -> Result<(), String> {
                AdminService::reload_from_config(self)
            }
        }
    };
    ($service:ty, $trait:ty, $config:ty, with_current_provider) => {
        impl $trait for $service {
            fn list_providers(&self) -> Vec<ProviderInfo> {
                AdminService::list_providers(self)
            }

            fn current_provider(&self) -> String {
                self.handle.provider_name()
            }

            fn switch_provider(&self, config: $config) -> Result<(), String> {
                AdminService::switch_provider(self, &config)
            }

            fn reload_from_config(&self) -> Result<(), String> {
                AdminService::reload_from_config(self)
            }
        }
    };
}
