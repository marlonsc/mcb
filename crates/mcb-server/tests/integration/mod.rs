//! Integration tests — `cargo test -p mcb-server --test integration`
#![allow(missing_docs)]

extern crate mcb_infrastructure;
extern crate mcb_providers;

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

pub mod handlers;
pub mod tools;

mod error_recovery_integration;
mod error_shape_tests;
mod form_deserialization_test;
mod full_stack_integration;
mod golden_acceptance_integration;
mod hooks_integration;

mod admin_api_tests;
mod auto_context_tests;
