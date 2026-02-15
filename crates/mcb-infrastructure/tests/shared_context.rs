#![allow(missing_docs)]
// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};
use std::sync::OnceLock;

static SHARED_APP_CONTEXT: OnceLock<AppContext> = OnceLock::new();

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
                    "default".to_string(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_string(),
                        path: Some(temp_path),
                    },
                );
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
