//! Shared `AppContext` for infrastructure test performance.
//!
//! Loads the ONNX embedding model once, reuses across all tests in the binary.
//! Included via `#[path]` in both unit and integration test roots.

// This module is shared between multiple test binaries;
// not all items are used in each.
#![allow(dead_code)]

const _: fn() = mcb_infrastructure::provider_linker::ensure_linked;

mcb_infrastructure::define_shared_test_context!("mcb-infra-shared.db");
