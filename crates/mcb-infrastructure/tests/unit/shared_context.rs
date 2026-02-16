//! Shared `AppContext` for infrastructure unit-test performance.
//!
//! Loads the ONNX embedding model once, reuses across all unit tests.

// Force linkme registration of all providers
extern crate mcb_providers;

use std::path::PathBuf;
use std::sync::OnceLock;

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};

/// Process-wide shared `AppContext`, or `None` when the ONNX model is
/// unavailable (offline / CI without model cache).
///
/// # Panics
///
/// Panics if `init_app` fails for reasons other than a missing ONNX model.
pub fn try_shared_app_context() -> Option<&'static AppContext> {
    static CTX: OnceLock<Option<AppContext>> = OnceLock::new();

    CTX.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let result = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_root = temp_dir.keep();
                let temp_path = temp_root.join("mcb-infra-unit-shared.db");

                let mut config = ConfigLoader::new().load().expect("load config");
                config.providers.database.configs.insert(
                    "default".to_owned(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_owned(),
                        path: Some(temp_path),
                    },
                );
                config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
                init_app(config).await
            });
            // Leak the runtime so it persists for the duration of the test process.
            // Otherwise, dropping `rt` kills all spawned background tasks/actors.
            std::mem::forget(rt);

            match result {
                Ok(ctx) => Some(ctx),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("model.onnx") || msg.contains("Failed to initialize FastEmbed")
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
/// Panics if the shared `AppContext` could not be initialized (ONNX model unavailable).
pub fn shared_app_context() -> &'static AppContext {
    try_shared_app_context().expect("shared AppContext init failed — ONNX model may be unavailable")
}

/// Persistent shared cache dir for the `FastEmbed` ONNX model.
///
/// Ensures the model is downloaded once and reused across test invocations.
///
/// # Panics
///
/// Panics if the cache directory cannot be created.
pub fn shared_fastembed_test_cache_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR").map_or_else(
            || std::env::temp_dir().join("mcb-fastembed-test-cache"),
            PathBuf::from,
        );
        std::fs::create_dir_all(&cache_dir).expect("create shared fastembed test cache dir");
        cache_dir
    })
    .clone()
}
