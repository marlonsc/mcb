//! Shared `AppContext` for infrastructure unit-test performance.
//!
//! Loads the ONNX embedding model once, reuses across all unit tests.

// Force linkme registration of all providers
extern crate mcb_providers;

mcb_infrastructure::define_shared_test_context!("mcb-infra-unit-shared.db");
