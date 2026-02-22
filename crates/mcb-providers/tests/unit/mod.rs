//! Unit test suite for mcb-providers
//!
//! Run with: `cargo test -p mcb-providers --test unit`

#[path = "../utils/mod.rs"]
mod utils;

mod analysis;
mod database;
mod events;
mod hybrid_search;
mod project_detection;
mod vcs;
mod workflow;
