//! Shared `AppContext` for infrastructure integration-test performance.
//!
//! Loads the ONNX embedding model once, reuses across all integration tests.

// This module is shared between `integration.rs` and `lib.rs` test binaries;
// not all functions are used in every binary.
#![allow(dead_code)]

// Force linkme registration of all providers
extern crate mcb_providers;

mcb_infrastructure::define_shared_test_context!("mcb-infra-integration-shared.db");
