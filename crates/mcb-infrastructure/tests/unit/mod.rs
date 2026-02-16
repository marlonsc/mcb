//! Unit test suite for mcb-infrastructure
//!
//! Run with: `cargo test -p mcb-infrastructure --test unit`
//!

// Shared test utilities
#[path = "../utils/mod.rs"]
mod test_utils;

mod shared_context;

pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error;
pub mod events;
pub mod health;
pub mod infrastructure;
pub mod logging;
pub mod routing;
pub mod services;
pub mod utils;
pub mod validation;
