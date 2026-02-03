//! Unit test suite for mcb-ast-utils
//!
//! Run with: `cargo test -p mcb-ast-utils --test unit`

#[path = "unit/complexity_tests.rs"]
mod complexity;

#[path = "unit/cursor_tests.rs"]
mod cursor;

#[path = "unit/visitor_tests.rs"]
mod visitor;

#[path = "unit/walker_tests.rs"]
mod walker;

#[path = "unit/symbols_tests.rs"]
mod symbols;
