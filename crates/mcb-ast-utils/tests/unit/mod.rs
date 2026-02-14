//! Unit test suite for mcb-ast-utils
//!
//! Run with: `cargo test -p mcb-ast-utils --test unit`

#[path = "common.rs"]
mod common;

#[path = "complexity_tests.rs"]
mod complexity;

#[path = "cursor_tests.rs"]
mod cursor;

#[path = "visitor_tests.rs"]
mod visitor;

#[path = "walker_tests.rs"]
mod walker;

#[path = "symbols_tests.rs"]
mod symbols;
