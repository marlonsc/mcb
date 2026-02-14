//! Shared AppContext for test performance
//!
//! Initializes the application context (including FastEmbed ONNX model) exactly
//! once per test binary, then shares it across all tests that need it.
//! This avoids the ~5-10s model load per test.

// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};
use std::sync::OnceLock;

static SHARED_APP_CONTEXT: OnceLock<AppContext> = OnceLock::new();

/// Returns a process-wide shared `AppContext`.
///
/// Spawns a dedicated thread with its own tokio runtime for initialization.
/// The runtime is intentionally **leaked** so it—and the actor tasks it
/// spawned—live for the entire process.  If we let the runtime drop, all
/// actors would be cancelled and subsequent tests would see "Actor closed".
///
/// Using `tokio::sync::OnceCell` instead of `std::sync::OnceLock` would
/// bind actors to the first `#[tokio::test]` runtime; when that test
/// finishes the runtime drops and actors die.
pub fn shared_app_context() -> &'static AppContext {
    SHARED_APP_CONTEXT.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let ctx = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_path = temp_dir.path().join("shared-test.db");
                std::mem::forget(temp_dir); // leak so path stays valid

                let mut config = ConfigLoader::new().load().expect("load config");
                config.auth.user_db_path = Some(temp_path);
                init_app(config)
                    .await
                    .expect("shared init_app should succeed")
            });
            // Leak the runtime so the actor tasks it spawned stay alive.
            std::mem::forget(rt);
            ctx
        })
        .join()
        .expect("init thread panicked")
    })
}
