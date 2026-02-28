//! Unit tests — `cargo test -p mcb-validate --test unit`
#![allow(missing_docs)]

#[path = "../utils/mod.rs"]
pub mod utils;

pub mod ast;
pub mod common;
pub mod engines;
pub mod filters;
pub mod linters;
pub mod rules;
pub mod scan;
pub mod util_tests;
pub mod validators;

mod declarative_validator_tests;
mod embedded_rules_tests;
mod lib_tests;
mod run_context_tests;
mod unified_registry_tests;
