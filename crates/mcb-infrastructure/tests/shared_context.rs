#![allow(missing_docs)]
// Force linkme registration of all providers
extern crate mcb_providers;

use std::path::PathBuf;
use std::sync::OnceLock;

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};

static SHARED_APP_CONTEXT: OnceLock<AppContext> = OnceLock::new();

/// Process-wide shared `AppContext` for infrastructure integration tests.
///
/// Loads the ONNX embedding model once, reuses across all tests.
/// The Tokio runtime is intentionally leaked so background tasks survive.
pub fn shared_app_context() -> &'static AppContext {
    SHARED_APP_CONTEXT.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let ctx = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_path = temp_dir.path().join("shared-infra-test.db");
                std::mem::forget(temp_dir);

                let mut config = ConfigLoader::new().load().expect("load config");
                config.providers.database.configs.insert(
                    "default".to_owned(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_owned(),
                        path: Some(temp_path),
                    },
                );
                config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
                init_app(config)
                    .await
                    .expect("shared init_app should succeed")
            });
            std::mem::forget(rt);
            ctx
        })
        .join()
        .expect("init thread panicked")
    })
}

/// Persistent shared cache dir for `FastEmbed` ONNX model.
///
/// Avoids re-downloading the model on every test invocation by using a
/// process-wide directory outside the per-test temp dirs.
fn shared_fastembed_test_cache_dir() -> PathBuf {
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
