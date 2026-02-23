//! DI Layer Tests
//!
//! Tests for the dependency injection layer following Clean Architecture principles.
//! These tests validate:
//! - Manual composition root (`AppContext` via `init_app`)
//! - Module composition
//! - Provider resolution (`linkme` registry)
//! - Architecture validation and bypass detection

mod architecture_validation_tests;
mod dispatch_tests;
mod handle_tests;
mod modules_tests;
mod resolver_tests;
