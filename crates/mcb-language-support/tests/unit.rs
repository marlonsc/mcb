//! Unit test suite for mcb-language-support
//!
//! Run with: `cargo test -p mcb-language-support --test unit`

#[path = "unit/detection_tests.rs"]
mod detection;

#[path = "unit/language_tests.rs"]
mod language;

#[path = "unit/chunking_tests.rs"]
mod chunking;

#[path = "unit/parser_tests.rs"]
mod parser;
