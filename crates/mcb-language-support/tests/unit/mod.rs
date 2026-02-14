//! Unit test suite for mcb-language-support
//!
//! Run with: `cargo test -p mcb-language-support --test unit`

#[path = "detection_tests.rs"]
mod detection;

#[path = "language_tests.rs"]
mod language;

#[path = "chunking_tests.rs"]
mod chunking;

#[path = "parser_tests.rs"]
mod parser;
