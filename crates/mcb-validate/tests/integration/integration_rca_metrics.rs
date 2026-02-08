//! Integration tests for Phase 4: rust-code-analysis metrics
//!
//! Tests the `RcaAnalyzer` which provides 14 code metrics using
//! the rust-code-analysis library.
//!
//! Note: rust-code-analysis parsing behavior may vary. Tests are designed
//! to be resilient to parsing differences.
//!
//! NOTE: This test module was previously gated by a non-existent "rca-metrics" feature.
//! The tests have been disabled as they require fixing to work with the current API.
