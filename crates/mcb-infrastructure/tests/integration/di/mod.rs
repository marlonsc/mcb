//! DI Layer Tests
//!
//! Tests for the dependency injection layer following Clean Architecture principles.
//! These tests validate:
//! - Component registration and resolution (via dill Catalog)
//! - Container bootstrap
//! - Module composition
//! - Provider resolution
//! - Architecture validation and bypass detection

mod architecture_validation_tests;
mod catalog_tests;
mod dispatch_tests;
mod handle_tests;
mod modules_tests;
mod resolver_tests;
