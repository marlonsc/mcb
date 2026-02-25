//! Common macros for infrastructure layer.

/// Generate shared resolver boilerplate: `new(config)`, `Debug` impl.
///
/// Used by `di/provider_resolvers.rs` for `EmbeddingProviderResolver`,
/// `VectorStoreProviderResolver`, and similar resolver structs.
macro_rules! impl_resolver_common {
    ($resolver:ident) => {
        impl $resolver {
            #[must_use]
            /// Creates a new resolver with the provided application config.
            pub fn new(config: Arc<AppConfig>) -> Self {
                Self { config }
            }
        }

        impl std::fmt::Debug for $resolver {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($resolver)).finish()
            }
        }
    };
}

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
        pub fn try_shared_app_context() -> Option<&'static $crate::di::bootstrap::AppContext> {
            static CTX: std::sync::OnceLock<Option<$crate::di::bootstrap::AppContext>> =
                std::sync::OnceLock::new();

            CTX.get_or_init(|| {
                std::thread::spawn(|| {
                    let rt = tokio::runtime::Runtime::new()
                        .unwrap_or_else(|e| panic!("create init runtime: {e}"));
                    let result = rt.block_on(async {
                        let (config, _temp_dir) = $crate::config::TestConfigBuilder::new()?
                            .with_temp_db($db_name)?
                            .with_fastembed_shared_cache()?
                            .build()?;

                        if let Some(td) = _temp_dir {
                            let _ = td.keep();
                        }

                        $crate::di::bootstrap::init_app(config).await
                    });
                    let _rt = std::mem::ManuallyDrop::new(rt);

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
                .unwrap_or_else(|_| {
                    eprintln!(
                        "WARN: shared context init thread panicked \
                         (ONNX/ort model likely unavailable — skipping dependent tests)"
                    );
                    None
                })
            })
            .as_ref()
        }

        #[allow(clippy::panic)]
        pub fn shared_app_context() -> &'static $crate::di::bootstrap::AppContext {
            try_shared_app_context()
                .expect("shared AppContext init failed — ONNX model may be unavailable")
        }

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
