//! Unit tests — `cargo test -p mcb-infrastructure --test unit`

// linkme force-link only — DO NOT use for type/function imports (CA019 enforced)
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
