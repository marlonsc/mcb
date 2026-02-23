//! Shared `AppContext` for mcb-application unit-test performance.
//!
//! Initializes the application context (including `FastEmbed` ONNX model) exactly
//! once per test binary, then shares it across all tests that need it.
//! This avoids the ~5-10s model load per test.

const _: fn() = mcb_infrastructure::provider_linker::ensure_linked;

mcb_infrastructure::define_shared_test_context!("mcb-app-unit-shared.db");
