use std::sync::OnceLock;

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};

/// Process-wide shared AppContext for unit tests.
///
/// Loads the ONNX embedding model exactly once and reuses across all unit
/// tests in this binary, eliminating redundant ~7s ONNX loads per test.
pub fn shared_app_context() -> &'static AppContext {
    static CTX: OnceLock<AppContext> = OnceLock::new();
    CTX.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let ctx = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_path = temp_dir.path().join("mcb-server-unit-shared.db");
                std::mem::forget(temp_dir);

                let mut config = ConfigLoader::new().load().expect("load config");
                config.providers.database.configs.insert(
                    "default".to_string(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_string(),
                        path: Some(temp_path),
                    },
                );
                config.providers.embedding.cache_dir = Some(shared_fastembed_cache_dir());
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

/// Persistent shared cache dir for FastEmbed ONNX model.
///
/// Avoids re-downloading the model on every test invocation by using a
/// process-wide directory outside the per-test temp dirs.
fn shared_fastembed_cache_dir() -> std::path::PathBuf {
    static DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("mcb-fastembed-test-cache"));
        std::fs::create_dir_all(&cache_dir).expect("create shared fastembed test cache dir");
        cache_dir
    })
    .clone()
}
