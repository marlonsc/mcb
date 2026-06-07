//! Unit tests — `cargo test -p mcb-infrastructure --test unit`

// Force-link mcb-validate so that its linkme-registered validators
// populate the VALIDATOR_ENTRIES distributed slice.
extern crate mcb_validate;

pub mod config;
pub mod constants;
pub mod crypto;
pub mod error;
pub mod events;
pub mod infrastructure;

pub mod routing;
pub mod services;
pub mod validation;
