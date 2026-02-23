//! Shared `AppContext` for mcb-application unit-test performance.
//!
//! Initializes the application context (including `FastEmbed` ONNX model) exactly
//! once per test binary, then shares it across all tests that need it.
//! This avoids the ~5-10s model load per test.

// Force linkme registration of all providers
extern crate mcb_providers;

mcb_infrastructure::define_shared_test_context!("mcb-app-unit-shared.db");
