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

// ============================================================================
// Shared Test Context
// ============================================================================

/// Generate shared `AppContext` test infrastructure for a test binary.
///
/// Expands to `try_shared_app_context()`, `shared_app_context()`, and
/// `shared_fastembed_test_cache_dir()` functions backed by process-wide
/// `OnceLock` statics so the ONNX embedding model is loaded only once.
///
/// ## Usage
///
/// ```ignore
/// extern crate mcb_providers;
/// mcb_infrastructure::define_shared_test_context!("my-test.db");
/// ```
#[macro_export]
macro_rules! define_shared_test_context {
    ($db_name:literal) => {
        /// Process-wide shared `AppContext`, or `None` when the ONNX model is
        /// unavailable (offline / CI without model cache).
        ///
        /// # Panics
        ///
        /// Panics if `init_app` fails for reasons other than a missing ONNX model.
        pub fn try_shared_app_context() -> Option<&'static $crate::di::bootstrap::AppContext> {
            static CTX: std::sync::OnceLock<Option<$crate::di::bootstrap::AppContext>> =
                std::sync::OnceLock::new();

            CTX.get_or_init(|| {
                std::thread::spawn(|| {
                    let rt = tokio::runtime::Runtime::new().expect("create init runtime");
                    let result = rt.block_on(async {
                        let temp_dir = tempfile::tempdir().expect("create temp dir");
                        let temp_root = temp_dir.keep();
                        let temp_path = temp_root.join($db_name);

                        let mut config = $crate::config::ConfigLoader::new()
                            .load()
                            .expect("load config");
                        config.providers.database.configs.insert(
                            "default".to_owned(),
                            $crate::config::DatabaseConfig {
                                provider: "sqlite".to_owned(),
                                path: Some(temp_path),
                            },
                        );
                        config.providers.embedding.cache_dir =
                            Some(shared_fastembed_test_cache_dir());
                        $crate::di::bootstrap::init_app(config).await
                    });
                    // Leak the runtime so background actors survive the test process.
                    std::mem::forget(rt);

                    match result {
                        Ok(ctx) => Some(ctx),
                        Err(e) => {
                            let msg = e.to_string();
                            if msg.contains("model.onnx")
                                || msg.contains("Failed to initialize FastEmbed")
                            {
                                None
                            } else {
                                panic!("shared init_app failed: {e}");
                            }
                        }
                    }
                })
                .join()
                .expect("init thread panicked")
            })
            .as_ref()
        }

        /// Convenience wrapper that panics when the ONNX model is unavailable.
        ///
        /// # Panics
        ///
        /// Panics if the shared `AppContext` could not be initialized.
        pub fn shared_app_context() -> &'static $crate::di::bootstrap::AppContext {
            try_shared_app_context()
                .expect("shared AppContext init failed — ONNX model may be unavailable")
        }

        /// Persistent shared cache dir for the `FastEmbed` ONNX model.
        ///
        /// Ensures the model is downloaded once and reused across test invocations.
        ///
        /// # Panics
        ///
        /// Panics if the cache directory cannot be created.
        pub fn shared_fastembed_test_cache_dir() -> std::path::PathBuf {
            static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
            DIR.get_or_init(|| {
                let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR").map_or_else(
                    || std::env::temp_dir().join("mcb-fastembed-test-cache"),
                    std::path::PathBuf::from,
                );
                std::fs::create_dir_all(&cache_dir)
                    .expect("create shared fastembed test cache dir");
                cache_dir
            })
            .clone()
        }
    };
}
