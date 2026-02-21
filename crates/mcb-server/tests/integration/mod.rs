//! Integration tests — `cargo test -p mcb-server --test integration`

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

pub mod admin;
pub mod handlers;
pub mod tools;

mod admin_api_integration;
mod axum_harness_smoke;
mod axum_health_integration;
mod browse_api_integration;
mod error_recovery_integration;
mod error_shape_tests;
mod full_stack_integration;
mod golden_acceptance_integration;
mod hooks_integration;
mod operating_modes_integration;
